use crate::api::{ChatEvent, GenericIncomingEvent, IncomingEvent, ReadPayload, StoreChatMessage};
use actix::{Actor, Arbiter, AsyncContext, Context, Handler, Message};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use lemmy_db_schema::{
  newtypes::{ChatRoomId, LocalUserId, PaginationCursor},
  source::{
    chat_message::{ChatMessage, ChatMessageInsertForm},
    chat_room::ChatRoom,
    last_read::LastRead,
  },
  utils::{ActualDbPool, DbPool},
};
use lemmy_db_views_chat::api::ChatMessagesResponse;
use lemmy_db_views_chat::api::LastReadResponse;
use lemmy_db_views_chat::api::PeerReadResponse;

use crate::bridge_message::{BridgeMessage, OutboundMessage};
use crate::broker::connect_now::ConnectNow;
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use crate::broker::presence_manager::PresenceManager;
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};
use lemmy_utils::redis::RedisClient;
use phoenix_channels_client::{url::Url, Channel, ChannelStatus, Event, Payload, Socket};
use std::{collections::HashMap, sync::Arc, time::Duration};
use chrono::{DateTime, Utc};
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
#[rtype(result = "FastJobResult<LastReadResponse>")]
pub struct GetLastRead {
  pub room_id: ChatRoomId,
  pub local_user_id: LocalUserId,
}
#[derive(Message)]
#[rtype(result = "FastJobResult<PeerReadResponse>")]
pub struct GetPeerRead {
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
    room_id: ChatRoomId,
    payload_opt: Option<ReadPayload>,
    read_at: Option<DateTime<Utc>>,
  ) -> FastJobResult<()> {
    // ---- Validate payload ----
    let payload = match payload_opt {
      Some(p) => p,
      None => {
        tracing::warn!("chat:read missing payload");
        return Ok(());
      }
    };

    let reader_id = payload.reader_id;
    if reader_id.0 == 0 {
      return Ok(()); // Ignore anonymous / invalid reader
    }

    let last_read = payload.last_read_message_id.clone();
    if last_read.0.is_empty() {
      tracing::warn!("chat:read missing last_read_message_id");
      return Ok(());
    }

    // ---- Update in-memory cache ----
    self
      .last_read
      .insert((room_id.clone(), reader_id), last_read.0.clone());

    // ---- Spawn async DB update ----
    {
      let pool = self.pool.clone();
      let room = room_id.clone();
      let last_read_for_db = last_read.clone();

      tokio::spawn(async move {
        let mut db = DbPool::Pool(&pool);
        if let Err(e) = LastRead::upsert(&mut db, reader_id, room, last_read_for_db.clone(), read_at).await {
          tracing::warn!("last_read upsert failed: {}", e);
        }
      });
    }

    tracing::debug!(
      "READ-ACK recv room={} reader={:?} last_id={}",
      room_id,
      reader_id,
      last_read.0
    );

    // ---- Broadcast to others (async) ----
    {
      let manager = self.clone();
      let room = room_id.clone();
      let payload_clone = payload.clone();

      tokio::spawn(async move {
        let _ = manager.broadcast_read_event(room, payload_clone).await;
      });
    }

    Ok(())
  }

  /// Re-broadcast a normalized `chat:read` event to local WS subscribers and Phoenix channel
  #[allow(dead_code)]
  async fn broadcast_read_event(
    &self,
    room_id: ChatRoomId,
    payload: ReadPayload,
  ) -> Result<(), FastJobError> {
    // Local broker broadcast to other clients on this node via BridgeMessage(AnyIncomingEvent::Read)
    let wrapped = GenericIncomingEvent::<ReadPayload> {
      event: ChatEvent::Read,
      room_id: room_id.clone(),
      topic: format!("room:{}", room_id),
      payload: Some(payload.clone()),
    };
    let json_payload = if let Some(m) = wrapped.payload {
      serde_json::to_value(&m).unwrap_or(serde_json::Value::Null)
    } else {
      serde_json::Value::Null
    };

    let out_event = IncomingEvent {
      room_id: room_id.clone(),
      event: wrapped.event,
      topic: format!("room:{}", room_id),
      payload: json_payload,
    };

    let bridge = OutboundMessage { out_event };
    self.issue_async::<SystemBroker, _>(bridge.clone());

    // Phoenix channel cast (for cross-node subscribers)
    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);
    let channel_name = format!("room:{}", room_id);
    let outbound_event_for_cast = ChatEvent::Read.to_string_value();
    let content_json = serde_json::to_vec(&payload).unwrap_or_default();
    Arbiter::current().spawn(async move {
      if let Ok(arc_chan) = get_or_create_channel(channels, socket, &channel_name).await {
        if let Ok(status) = arc_chan.statuses().status().await {
          let phoenix_event = Event::from_string(outbound_event_for_cast);
          let payload: Payload = Payload::binary_from_bytes(content_json);
          if status == ChannelStatus::Joined {
            send_event_to_channel(arc_chan, phoenix_event, payload).await;
          } else {
            let _ = arc_chan.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await;
            send_event_to_channel(arc_chan, phoenix_event, payload).await;
          }
        }
      }
    });
    Ok(())
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
    new_message: Option<ChatMessageInsertForm>,
  ) -> FastJobResult<()> {
    let room_id = new_message.clone().unwrap().room_id;
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
  pub async fn drain_room_buffer(
    &mut self,
    room_id: &ChatRoomId,
  ) -> FastJobResult<Vec<ChatMessageInsertForm>> {
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
      tracing::info!(
        "Drained {} messages from room {} in Redis",
        messages.len(),
        room_id
      );
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
        let active_rooms_key = ChatEvent::ActiveRooms.to_string_value();

        // Fetch active rooms
        let raw_room_ids: Vec<String> = match redis.smembers(&active_rooms_key).await {
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
          if let Err(e) = redis.srem(&active_rooms_key, room_id.to_string()).await {
            if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
              tracing::error!("Failed to remove room {} from active rooms: {}", room_id, e);
            }
          }
        }

        // Set expiration for the active rooms set
        if let Err(e) = redis.expire(&active_rooms_key, MESSAGE_EXPIRY_SECS).await {
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
    let message = msg.message.unwrap();
    let mut this = self.clone();

    actix::spawn(async move {
      if let Err(e) = this.add_messages_to_room(Some(message)).await {
        tracing::error!("Failed to store message in Redis: {}", e);
      }
    });
  }
}
