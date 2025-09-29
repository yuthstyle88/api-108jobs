use crate::{
  bridge_message::{BridgeMessage, OnlineJoin, OnlineLeave},
  message::{RegisterClientMsg, StoreChatMessage},
};
use actix::{Actor, AsyncContext, Context, Handler, Message, ResponseFuture};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use chrono::Utc;
use lemmy_db_schema::source::chat_participant::{ChatParticipant, ChatParticipantInsertForm};
use lemmy_db_schema::{
  newtypes::{ChatMessageRefId, ChatRoomId, LocalUserId, PaginationCursor},
  source::{
    chat_message::{ChatMessage, ChatMessageInsertForm},
    chat_room::ChatRoom,
    last_read::LastRead,
  },
  traits::PaginationCursorBuilder,
  utils::{ActualDbPool, DbPool},
};
use lemmy_db_views_chat::api::ChatMessagesResponse;

use crate::broker::connect_now::ConnectNow;
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use lemmy_db_views_chat::ChatMessageView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use phoenix_channels_client::{url::Url, Channel, ChannelStatus, Event, Payload, Socket, Topic};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

// Timeouts and intervals (in seconds) for Phoenix socket/channel operations
pub const CONNECT_TIMEOUT_SECS: u64 = 10;
pub const JOIN_TIMEOUT_SECS: u64 = 5;
pub const FLUSH_INTERVAL_SECS: u64 = 10;


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

pub struct PhoenixManager {
  pub(crate) socket: Arc<Socket>,
  pub(crate) channels: Arc<RwLock<HashMap<String, Arc<Channel>>>>,
  chat_store: HashMap<ChatRoomId, Vec<ChatMessageInsertForm>>,
  pub(crate) pool: ActualDbPool,
  is_flushing: bool,
  pub(crate) last_read: HashMap<(ChatRoomId, LocalUserId), String>,
  pub(crate) online_counts: HashMap<(ChatRoomId, LocalUserId), usize>,
}

impl PhoenixManager {
  pub async fn new(endpoint: &Option<Url>, pool: ActualDbPool) -> Self {
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
      chat_store: HashMap::new(),
      pool,
      is_flushing: false,
      last_read: HashMap::new(),
      online_counts: Default::default(),
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
    self.broadcast_read_event(&chatroom_id, reader_id_val, &last_read_id, msg);
  }

  /// Re-broadcast a normalized `chat:read` event to local WS subscribers and Phoenix channel
  fn broadcast_read_event(
    &self,
    chatroom_id: &ChatRoomId,
    reader_id_val: i64,
    last_read_id: &str,
    msg: &BridgeMessage,
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
      local_user_id: msg.local_user_id.clone(),
      event: outbound_event.clone(),
      messages: content.clone(),
      security_config: false,
    });

    // Phoenix channel cast (for cross-node subscribers)
    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);
    let channel_name = format!("room:{}", outbound_channel);
    let outbound_event_for_cast = outbound_event.clone();
    actix::spawn(async move {
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

  pub fn add_messages_to_room(&mut self, room_id: ChatRoomId, new_messages: ChatMessageInsertForm) {
    self
      .chat_store
      .entry(room_id)
      .or_default()
      .push(new_messages);
  }

  /// Drain buffered messages for a room and return them for persistence (non-blocking on the actor)
  pub(crate) fn drain_room_buffer(&mut self, room_id: &ChatRoomId) -> Vec<ChatMessageInsertForm> {
    if let Some(buffer) = self.chat_store.get_mut(room_id) {
      buffer.drain(..).collect()
    } else {
      Vec::new()
    }
  }

  // Update a message in the chat store for a specific room
  #[allow(dead_code)] // used in upcoming WebSocket message sync logic
  fn update_chat_message(
    &mut self,
    room_id: &ChatRoomId,
    predicate: impl Fn(&ChatMessageInsertForm) -> bool,
    update_fn: impl FnOnce(&mut ChatMessageInsertForm),
  ) {
    if let Some(messages) = self.chat_store.get_mut(room_id) {
      if let Some(message) = messages.iter_mut().find(|msg| predicate(msg)) {
        update_fn(message);
      }
    }
  }

  pub(crate) fn ensure_room_initialized(&mut self, room_id: ChatRoomId, _room_name: String) {
    if !self.chat_store.contains_key(&room_id) {
      // ensure in-memory buffer for this room exists, but do NOT create DB room here
      self.chat_store.insert(room_id.clone(), Vec::new());
    }
  }
}

impl Actor for PhoenixManager {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    ctx.notify(ConnectNow);
    self.subscribe_system_async::<BridgeMessage>(ctx);
    ctx.run_interval(Duration::from_secs(FLUSH_INTERVAL_SECS), |actor, ctx| {
      if actor.is_flushing {
        // Skip this tick if a previous flush is still running
        return;
      }
      actor.is_flushing = true;

      let drained = std::mem::take(&mut actor.chat_store);
      let pool = actor.pool.clone();
      let addr = ctx.address();

      actix::spawn(async move {
        for (room_id, messages) in drained.into_iter() {
          if messages.is_empty() {
            continue;
          }
          tracing::info!("Flushing {} messages from room {}", messages.len(), room_id);
          let mut db_pool = DbPool::Pool(&pool);
          if let Err(e) = ChatMessage::bulk_insert(&mut db_pool, &messages).await {
            tracing::error!("Failed to flush messages: {}", e);
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
    let msg = msg.message;
    self
      .chat_store
      .entry(msg.room_id.clone())
      .or_default()
      .push(msg);
  }
}

