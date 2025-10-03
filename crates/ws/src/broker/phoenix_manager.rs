use crate::message::StoreChatMessage;
use actix::{Actor, Arbiter, AsyncContext, Context, Handler, Message};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use lemmy_db_schema::{
  newtypes::{ChatMessageRefId, ChatRoomId, LocalUserId, PaginationCursor},
  source::{
    chat_message::{ChatMessage, ChatMessageInsertForm},
    chat_room::ChatRoom,
    last_read::LastRead,
  },
  utils::{ActualDbPool, DbPool},
};
use lemmy_db_views_chat::api::ChatMessagesResponse;

use crate::bridge_message::BridgeMessage;
use crate::broker::connect_now::ConnectNow;
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use crate::broker::presence_manager::PresenceManager;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use lemmy_utils::redis::RedisClient;
use phoenix_channels_client::{url::Url, Channel, ChannelStatus, Event, Payload, Socket};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

// Timeouts and intervals (in seconds) for Phoenix socket/channel operations
pub const CONNECT_TIMEOUT_SECS: u64 = 10;
pub const JOIN_TIMEOUT_SECS: u64 = 5;
pub const FLUSH_INTERVAL_SECS: u64 = 10;
pub const MESSAGE_EXPIRY_SECS: usize = 3600;

#[derive(Message)]
#[rtype(result = "()")]
struct FlushDone;

#[derive(Message)]
#[rtype(result = "Option<String>")]
pub struct GetLastRead {
  pub room_id: ChatRoomId,
  pub local_user_id: LocalUserId,
}

#[derive(Message)]
#[rtype(result = "FastJobResult<ChatMessagesResponse>")]
pub struct FetchHistoryDirect {
  pub local_user_id: LocalUserId,
  pub room_id: ChatRoomId,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
  pub page_back: Option<bool>,
}

#[derive(Clone)]
pub struct PhoenixManager {
  pub(crate) socket: Arc<Socket>,
  pub(crate) channels: Arc<RwLock<HashMap<String, Arc<Channel>>>>,
  pub(crate) presence: actix::Addr<PresenceManager>,
  pub(crate) pool: ActualDbPool,
  is_flushing: bool,
  pub(crate) last_read: HashMap<(ChatRoomId, LocalUserId), String>,
  pub(crate) redis_client: Arc<RedisClient>,
}

impl PhoenixManager {
  pub async fn new(
    endpoint: &Option<Url>,
    pool: ActualDbPool,
    presence: actix::Addr<PresenceManager>,
    redis_client: RedisClient,
  ) -> Self {
    let sock = Socket::spawn(
      endpoint.clone().expect("Phoenix url is require"),
      None,
      None,
    )
    .await
    .expect("Failed to create socket");
    Self {
      socket: sock,
      channels: Arc::new(RwLock::new(HashMap::new())),
      presence,
      pool,
      is_flushing: false,
      last_read: HashMap::new(),
      redis_client: Arc::new(redis_client),
    }
  }

  /// Persist (async) and broadcast a normalized read event; updates in-memory pointer as well.
  pub(crate) fn handle_read_event(
    &mut self,
    msg: &BridgeMessage,
    chatroom_id: ChatRoomId,
    obj: &serde_json::Map<String, serde_json::Value>,
  ) {
    // reader_id: รับได้ทั้ง number หรือ string
    let reader_id_val = obj
      .get("reader_id")
      .and_then(|v| {
        v.as_i64()
          .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
      })
      .unwrap_or_else(|| {
        tracing::warn!("chat:read missing/invalid reader_id");
        0
      });
    if reader_id_val == 0 {
      return;
    }

    // last_read_message_id: str เท่านั้น
    let last_read_id = match obj.get("last_read_message_id").and_then(|v| v.as_str()) {
      Some(s) if !s.is_empty() => s.to_string(),
      _ => {
        tracing::warn!("chat:read missing last_read_message_id");
        return;
      }
    };

    self.last_read.insert(
      (chatroom_id.clone(), LocalUserId(reader_id_val as i32)),
      last_read_id.clone(),
    );

    // upsert async (ไม่บล็อกเส้นทาง broadcast)
    let pool_for_last = self.pool.clone();
    let room_for_last = chatroom_id.clone();
    let reader_local = LocalUserId(reader_id_val as i32);
    let msg_id_wrap = ChatMessageRefId(last_read_id.clone());
    tokio::spawn(async move {
      let mut db = DbPool::Pool(&pool_for_last);
      if let Err(e) = LastRead::upsert(&mut db, reader_local, room_for_last, msg_id_wrap).await {
        tracing::warn!("last_read upsert failed: {}", e);
      }
    });

    tracing::debug!(
      "READ-ACK recv room={} reader={} last_id={}",
      chatroom_id,
      reader_id_val,
      last_read_id
    );

    let this = self.clone();
    let chatroom_id_cloned = chatroom_id.clone();
    let last_read_id_cloned = last_read_id.clone();
    let msg_cloned = msg.clone();
    tokio::spawn(async move {
      this
        .broadcast_read_event(
          &chatroom_id_cloned,
          reader_id_val as i32,
          &last_read_id_cloned,
          &msg_cloned,
        )
        .await;
    });
  }

  /// Re-broadcast a normalized `chat:read` event to local WS subscribers and Phoenix channel
  async fn broadcast_read_event(
    &self,
    chatroom_id: &ChatRoomId,
    reader_id_val: i32,
    last_read_id: &str,
    _msg: &BridgeMessage,
  ) {
    // Build flat payload
    let mut read_payload = serde_json::Map::new();
    read_payload.insert(
      "room_id".to_string(),
      serde_json::Value::String(chatroom_id.to_string()),
    );
    read_payload.insert(
      "reader_id".to_string(),
      serde_json::Value::Number(reader_id_val.into()),
    );
    read_payload.insert(
      "last_read_message_id".to_string(),
      serde_json::Value::String(last_read_id.to_string()),
    );
    let content = serde_json::Value::Object(read_payload).to_string();


    // Local broker broadcast (to other clients on this node)
    let outbound_channel = chatroom_id.clone();
    let outbound_event = "chat:read".to_string();
    self.issue_async::<SystemBroker, _>(BridgeMessage {
      channel: outbound_channel.clone(),
      event: outbound_event.clone(),
      messages: content.clone(),
      security_config: false,
    });

    // Phoenix channel cast (for cross-node subscribers)
    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);
    let channel_name = format!("room:{}", outbound_channel);
    let outbound_event_for_cast = outbound_event.clone();
    Arbiter::current().spawn(async move {
      if let Ok(arc_chan) = get_or_create_channel(channels, socket, &channel_name).await {
        if let Ok(status) = arc_chan.statuses().status().await {
          let phoenix_event = Event::from_string(outbound_event_for_cast);
          let payload: Payload = Payload::binary_from_bytes(content.into_bytes());
          if status == ChannelStatus::Joined {
            send_event_to_channel(arc_chan, phoenix_event, payload).await;
          } else {
            let _ = arc_chan.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await;
            send_event_to_channel(arc_chan, phoenix_event, payload).await;
          }
        }
      }
    });
  }

  pub async fn validate_or_create_room(
    &mut self,
    room_id: ChatRoomId,
    _room_name: String,
  ) -> FastJobResult<()> {
    let room_id_str = ChatRoomId::try_from(room_id)?;
    let mut db_pool = DbPool::Pool(&self.pool);
    if !ChatRoom::exists(&mut db_pool, room_id_str).await? {
      return Err(FastJobErrorType::NotFound.into());
    }
    Ok(())
  }

  /// Add a message to a room's message list in Redis
  pub async fn add_messages_to_room(
    &mut self,
    room_id: ChatRoomId,
    new_message: ChatMessageInsertForm,
  ) -> FastJobResult<()> {
    let key = format!("chat:room:{}:messages", room_id);
    let active_rooms_key = "chat:active_rooms";
    let mut redis = self.redis_client.as_ref().clone();

    // Append message to the Redis list
    redis.rpush(&key, &new_message).await?;

    // Set expiration for the room's message list
    redis.expire(&key, MESSAGE_EXPIRY_SECS).await?;

    // Add room to active rooms set
    redis.sadd(active_rooms_key, room_id.to_string()).await?;

    // Set expiration for the active rooms set
    redis.expire(active_rooms_key, MESSAGE_EXPIRY_SECS).await?;

    tracing::debug!("Added message to room {} in Redis", room_id);
    Ok(())
  }

  /// Drain buffered messages for a room from Redis and return them for persistence
  pub async fn drain_room_buffer(&mut self, room_id: &ChatRoomId) -> FastJobResult<Vec<ChatMessageInsertForm>> {
    let key = format!("chat:room:{}:messages", room_id);
    let mut redis = self.redis_client.as_ref().clone();

    // Fetch all messages from the Redis list
    let messages: Vec<ChatMessageInsertForm> = redis.lrange(&key, 0, -1).await?;

    // Delete the key (ignore RedisKeyNotFound error)
    if !messages.is_empty() {
      if let Err(e) = redis.delete_key(&key).await {
        if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
          tracing::error!("Failed to delete Redis key for room {}: {}", room_id, e);
        }
      }
      tracing::info!("Drained {} messages from room {} in Redis", messages.len(), room_id);
    }

    Ok(messages)
  }

  /// Update a message in the Redis store for a specific room
  #[allow(dead_code)]
  async fn update_chat_message(
    &mut self,
    room_id: &ChatRoomId,
    predicate: impl Fn(&ChatMessageInsertForm) -> bool,
    update_fn: impl FnOnce(&mut ChatMessageInsertForm),
  ) -> FastJobResult<()> {
    let key = format!("chat:room:{}:messages", room_id);
    let mut redis = self.redis_client.as_ref().clone();

    // Fetch all messages
    let mut messages: Vec<ChatMessageInsertForm> = redis.lrange(&key, 0, -1).await?;

    // Update matching message
    let found = messages.iter_mut().find(|msg| predicate(msg));
    if let Some(message) = found {
      update_fn(message);

      // Clear the existing list
      if let Err(e) = redis.delete_key(&key).await {
        if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
          tracing::error!("Failed to delete Redis key for room {}: {}", room_id, e);
        }
      }

      // Re-push updated messages
      for message in messages {
        redis.rpush(&key, &message).await?;
      }

      // Reset expiration
      redis.expire(&key, MESSAGE_EXPIRY_SECS).await?;

      tracing::debug!("Updated message in room {} in Redis", room_id);
    }

    Ok(())
  }
}

impl Actor for PhoenixManager {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    ctx.notify(ConnectNow);
    self.subscribe_system_async::<BridgeMessage>(ctx);
    ctx.run_interval(Duration::from_secs(FLUSH_INTERVAL_SECS), |actor, ctx| {
      if actor.is_flushing {
        return;
      }
      actor.is_flushing = true;

      let pool = actor.pool.clone();
      let redis_client = Arc::clone(&actor.redis_client);
      let addr = ctx.address();

      actix::spawn(async move {
        let mut redis = redis_client.as_ref().clone();
        let active_rooms_key = "chat:active_rooms";

        // Fetch active rooms
        let raw_room_ids: Vec<String> = match redis.smembers(active_rooms_key).await {
          Ok(rooms) => rooms,
          Err(e) => {
            tracing::error!("Failed to fetch active rooms: {}", e);
            addr.do_send(FlushDone);
            return;
          }
        };

        let mut db_pool = DbPool::Pool(&pool);
        for room_id_str in raw_room_ids {
          let room_id = match room_id_str.parse::<ChatRoomId>() {
            Ok(id) => id,
            Err(e) => {
              tracing::error!("Invalid room ID {}: {}", room_id_str, e);
              continue;
            }
          };

          let key = format!("chat:room:{}:messages", room_id);
          let messages: Vec<ChatMessageInsertForm> = match redis.lrange(&key, 0, -1).await {
            Ok(messages) => messages,
            Err(e) => {
              tracing::error!("Failed to fetch messages for room {}: {}", room_id, e);
              continue;
            }
          };

          if messages.is_empty() {
            continue;
          }

          tracing::info!("Flushing {} messages from room {}", messages.len(), room_id);
          if let Err(e) = ChatMessage::bulk_insert(&mut db_pool, &messages).await {
            tracing::error!("Failed to flush messages for room {}: {}", room_id, e);
            continue;
          }

          // Delete the room's messages and remove from active rooms
          if let Err(e) = redis.delete_key(&key).await {
            if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
              tracing::error!("Failed to delete Redis key for room {}: {}", room_id, e);
            }
          }
          if let Err(e) = redis.srem(active_rooms_key, room_id.to_string()).await {
            if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
              tracing::error!("Failed to remove room {} from active rooms: {}", room_id, e);
            }
          }
        }

        // Set expiration for the active rooms set
        if let Err(e) = redis.expire(active_rooms_key, MESSAGE_EXPIRY_SECS).await {
          if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
            tracing::error!("Failed to set expiration for active rooms: {}", e);
          }
        }

        addr.do_send(FlushDone);
      });
    });
  }
}

impl Handler<FlushDone> for PhoenixManager {
  type Result = ();
  fn handle(&mut self, _msg: FlushDone, _ctx: &mut Context<Self>) {
    self.is_flushing = false;
  }
}

impl Handler<StoreChatMessage> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: StoreChatMessage, _ctx: &mut Context<Self>) -> Self::Result {
    let message = msg.message;
    let room_id = message.room_id.clone();
    let mut this = self.clone();

    actix::spawn(async move {
      if let Err(e) = this.add_messages_to_room(room_id, message).await {
        tracing::error!("Failed to store message in Redis: {}", e);
      }
    });
  }
}
