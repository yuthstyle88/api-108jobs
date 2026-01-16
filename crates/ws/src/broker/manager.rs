use crate::protocol::api::{
  ChatEvent, GenericIncomingEvent, IncomingEvent, ReadPayload, StoreChatMessage,
};
use actix::{Actor, Arbiter, AsyncContext, Context, Handler, Message, ResponseFuture};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use app_108jobs_db_schema::{
  newtypes::{ChatRoomId, LocalUserId, PaginationCursor},
  source::{chat_message::ChatMessageInsertForm, chat_room::ChatRoom, last_read::LastRead},
  utils::{ActualDbPool, DbPool},
};
use app_108jobs_db_views_chat::api::ChatMessagesResponse;
use app_108jobs_db_views_chat::api::LastReadResponse;
use app_108jobs_db_views_chat::api::PeerReadResponse;
use app_108jobs_db_views_chat::api::UnreadSnapshotItem;

use crate::protocol::api::PresenceSnapshotItem;

use crate::bridge_message::{BridgeMessage, OutboundMessage};
use crate::broker::connect_now::ConnectNow;
use crate::presence::{IsUserOnline, PresenceManager};
use crate::protocol::phx_helper::{get_or_create_channel, send_event_to_channel};
use app_108jobs_api_utils::utils::flush_room_and_update_last_message;
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};
use app_108jobs_utils::redis::RedisClient;
use chrono::{DateTime, Utc};
use phoenix_channels_client::{url::Url, Channel, ChannelStatus, Event, Payload, Socket};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

// Timeouts and intervals (in seconds) for Phoenix socket/channel operations
pub const CONNECT_TIMEOUT_SECS: u64 = 10;
pub const JOIN_TIMEOUT_SECS: u64 = 5;
pub const FLUSH_INTERVAL_SECS: u64 = 10;
pub const MESSAGE_EXPIRY_SECS: usize = 3600;
pub const ACTIVE_ROOMS_KEY: &'static str = "chat:active_rooms";

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

#[derive(Message)]
#[rtype(result = "FastJobResult<Vec<UnreadSnapshotItem>>")]
pub struct GetUnreadSnapshot {
  pub local_user_id: LocalUserId,
}

#[derive(Message)]
#[rtype(result = "FastJobResult<Vec<PresenceSnapshotItem>>")]
pub struct GetPresenceSnapshot {
  pub local_user_id: LocalUserId,
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
        if let Err(e) =
          LastRead::upsert(&mut db, reader_id, room, last_read_for_db.clone(), read_at).await
        {
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
      self.try_delete_room_buffer(&mut redis, room_id).await?;
      tracing::info!(
        "Drained {} messages from room {} in Redis",
        messages.len(),
        room_id
      );
    }

    Ok(messages)
  }

  /// Main flush entry point — clean, testable, single source of truth
  pub async fn flush_all_buffered_messages(&self) -> FastJobResult<()> {
    let mut redis = self.redis_client.as_ref().clone();

    // 1. Get all active room IDs
    let room_ids = match self.load_active_room_ids(&mut redis).await {
      Ok(ids) => ids,
      Err(e) => {
        tracing::error!("Failed to load active rooms during flush: {e}");
        return Ok(()); // non-fatal, continue next cycle
      }
    };

    if room_ids.is_empty() {
      return Ok(());
    }

    let mut db = DbPool::Pool(&self.pool);

    // 2. Process each room
    for room_id in &room_ids {
      if let Err(e) = self
        .flush_single_room(&mut redis, &mut db, room_id.clone())
        .await
      {
        tracing::error!("Failed to flush room {room_id}: {e}");
      }
    }

    // 3. Refresh TTL on active rooms set
    let _ = redis.expire(ACTIVE_ROOMS_KEY, MESSAGE_EXPIRY_SECS).await;

    Ok(())
  }

  /// Load and parse active room IDs safely
  async fn load_active_room_ids(&self, redis: &mut RedisClient) -> FastJobResult<Vec<ChatRoomId>> {
    let raw: Vec<String> = redis.smembers(ACTIVE_ROOMS_KEY).await?;
    let ids = raw
      .into_iter()
      .filter_map(|s| s.parse::<ChatRoomId>().ok())
      .collect();
    Ok(ids)
  }

  /// Flush one room — atomic, safe, logs clearly
  async fn flush_single_room(
    &self,
    redis: &mut RedisClient,
    db: &mut DbPool<'_>,
    room_id: ChatRoomId,
  ) -> FastJobResult<()> {
    flush_room_and_update_last_message(db, redis, room_id).await
  }

  // Your preferred helpers — perfect as-is
  async fn try_delete_room_buffer(
    &self,
    redis: &mut RedisClient,
    room_id: &ChatRoomId,
  ) -> FastJobResult<()> {
    let key = format!("chat:room:{}:messages", room_id);
    if let Err(e) = redis.delete_key(&key).await {
      if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
        tracing::error!("Failed to delete Redis buffer for room {room_id}: {e}");
      }
    }
    Ok(())
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

    ctx.run_interval(Duration::from_secs(FLUSH_INTERVAL_SECS), move |act, ctx| {
      if act.is_flushing {
        return;
      }
      act.is_flushing = true;

      let manager = act.clone();
      let addr = ctx.address();

      actix::spawn(async move {
        if let Err(e) = manager.flush_all_buffered_messages().await {
          tracing::error!("Periodic message flush failed: {e}");
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

impl Handler<IsUserOnline> for PhoenixManager {
  type Result = ResponseFuture<bool>;

  fn handle(&mut self, msg: IsUserOnline, _ctx: &mut Context<Self>) -> Self::Result {
    let presence = self.presence.clone();
    Box::pin(async move {
      if let Ok(result) = presence.send(msg).await {
        return result;
      }
      false
    })
  }
}
