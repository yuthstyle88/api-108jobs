use crate::{
  bridge_message::BridgeMessage,
  message::{RegisterClientMsg, StoreChatMessage},
};
use actix::{Actor, AsyncContext, Context, Handler, Message, ResponseFuture};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use chrono::Utc;
use lemmy_db_schema::source::chat_participant::{ChatParticipant, ChatParticipantInsertForm};
use lemmy_db_schema::{
  newtypes::{ChatRoomId, LocalUserId, PaginationCursor},
  source::{
    chat_message::{ChatMessage, ChatMessageInsertForm},
    chat_room::ChatRoom,
  },
  traits::PaginationCursorBuilder,
  utils::{ActualDbPool, DbPool},
};
use lemmy_db_views_chat::api::ChatMessagesResponse;
use lemmy_db_views_chat::ChatMessageView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use phoenix_channels_client::{url::Url, Channel, ChannelStatus, Event, Payload, Socket, Topic};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

// Timeouts and intervals (in seconds) for Phoenix socket/channel operations
const CONNECT_TIMEOUT_SECS: u64 = 10;
const JOIN_TIMEOUT_SECS: u64 = 5;
const FLUSH_INTERVAL_SECS: u64 = 10;

#[derive(Message)]
#[rtype(result = "()")]
struct ConnectNow;

#[derive(Message)]
#[rtype(result = "()")]
struct FlushDone;

#[derive(Message)]
#[rtype(result = "FastJobResult<ChatMessagesResponse>")]
pub struct FetchHistoryDirect {
  pub user_id: LocalUserId,
  pub room_id: ChatRoomId,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
  pub page_back: Option<bool>,
}

async fn connect(socket: Arc<Socket>) -> FastJobResult<Arc<Socket>> {
  // Try to connect
  match socket
    .connect(Duration::from_secs(CONNECT_TIMEOUT_SECS))
    .await
  {
    Ok(_) => Ok(socket),
    Err(e) => {
      tracing::error!("Failed to connect to socket: {}", e);
      Err(e.into())
    }
  }
}
async fn send_event_to_channel(channel: Arc<Channel>, event: Event, payload: Payload) {
  if let Err(e) = channel.cast(event, payload).await {
    tracing::error!("Failed to cast message: {}", e);
  }
}
async fn get_or_create_channel(
  channels: Arc<RwLock<HashMap<String, Arc<Channel>>>>,
  socket: Arc<Socket>,
  name: &str,
) -> FastJobResult<Arc<Channel>> {
  // Try to get existing channel
  if let Some(channel) = channels.read().await.get(name).cloned() {
    match channel.statuses().status().await {
      Ok(status) => {
        if status == ChannelStatus::Joined {
          tracing::info!("Using existing channel: {}", name);
          return Ok(channel);
        }
        // Not joined; attempt to rejoin
        if let Ok(_) = channel.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await {
          tracing::info!("Successfully rejoined channel: {}", name);
          return Ok(channel);
        }
      }
      Err(e) => {
        tracing::info!("Channel {} status check failed: {}", name, e);
      }
    }
    channels.write().await.remove(name);
  }

  // Create new channel
  let topic = Topic::from_string(name.to_string());
  let channel = socket
    .channel(topic, None)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create channel {}: {}", name, e))?;

  // Join channel
  channel
    .join(Duration::from_secs(JOIN_TIMEOUT_SECS))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to join channel {}: {}", name, e))?;

  // Store new channel
  channels
    .write()
    .await
    .insert(name.to_string(), channel.clone());
  tracing::info!("Created new channel: {}", name);
  Ok(channel)
}

/// List chat messages using cursor pagination (module-level function; no coupling to PhoenixManager)
pub async fn list_chat_messages(
  pool: ActualDbPool,
  room_id: ChatRoomId,
  page_cursor: Option<PaginationCursor>,
  limit: Option<i64>,
  page_back: Option<bool>,
) -> FastJobResult<ChatMessagesResponse> {
  let mut db_pool = DbPool::Pool(&pool);

  // Decode cursor data if provided
  let cursor_data = if let Some(ref cursor) = page_cursor {
    Some(ChatMessageView::from_cursor(cursor, &mut db_pool).await?)
  } else {
    None
  };

  // Sanitize limit: default 20, clamp to 1..=100
  let mut lim = limit.unwrap_or(20);
  if lim <= 0 {
    lim = 20;
  }
  if lim > 100 {
    lim = 100;
  }
  let lim = Some(lim);

  // If a cursor exists and direction not specified, default to paging backward (older)
  let effective_page_back = match (cursor_data.as_ref(), page_back) {
    (Some(_), None) => Some(true),
    _ => page_back,
  };

  let results =
    ChatMessageView::list_for_room(&mut db_pool, room_id, lim, cursor_data, effective_page_back)
      .await?;

  let next_page = results.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = results.first().map(PaginationCursorBuilder::to_cursor);

  Ok(ChatMessagesResponse {
    results,
    next_page,
    prev_page,
  })
}

pub struct PhoenixManager {
  socket: Arc<Socket>,
  channels: Arc<RwLock<HashMap<String, Arc<Channel>>>>,
  chat_store: HashMap<ChatRoomId, Vec<ChatMessageInsertForm>>,
  pool: ActualDbPool,
  is_flushing: bool,
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
    }
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
  fn drain_room_buffer(&mut self, room_id: &ChatRoomId) -> Vec<ChatMessageInsertForm> {
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

  fn ensure_room_initialized(&mut self, room_id: ChatRoomId, _room_name: String) {
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

impl Handler<ConnectNow> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, _msg: ConnectNow, ctx: &mut Context<Self>) -> Self::Result {
    let socket = self.socket.clone();
    let addr = ctx.address();
    actix::spawn(async move {
      match connect(socket).await {
        Ok(sock) => {
          addr.do_send(InitSocket(sock));
        }
        Err(e) => {
          tracing::error!("Failed to connect Phoenix socket: {}", e);
        }
      }
    });
  }
}

// Handler for BridgeMessage
impl Handler<BridgeMessage> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: BridgeMessage, _ctx: &mut Context<Self>) -> Self::Result {
    // Process only messages coming from Phoenix client; ignore ones we ourselves rebroadcast to avoid loops

    let channel_name = msg.channel.to_string();
    let user_id = msg.user_id.clone();
    let event = msg.event.clone();

    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);
    let message = msg.messages.clone();

    let chatroom_id = ChatRoomId::from_channel_name(channel_name.as_str())
      .unwrap_or_else(|_| ChatRoomId(channel_name.strip_prefix("room:").unwrap_or(&channel_name).to_string()));

    // Parse incoming JSON payload (may be object/array/string). We expect an object for send_message.
    let incoming_val: serde_json::Value =
      serde_json::from_str(&message).unwrap_or_else(|_| serde_json::Value::Null);
    let obj = match incoming_val {
      serde_json::Value::Object(map) => map,
      _ => serde_json::Map::new(),
    };

    // Extract fields with sensible fallbacks
    let content_text = obj
      .get("content")
      .and_then(|v| v.as_str())
      .unwrap_or_else(|| message.as_str());
    let room_id_str: String = obj
      .get("room_id")
      .and_then(|v| v.as_str().map(|s| s.to_string()))
      .or_else(|| {
        obj
          .get("roomId")
          .and_then(|v| v.as_str().map(|s| s.to_string()))
      })
      .unwrap_or_else(|| chatroom_id.to_string());
    let sender_id_val = obj
      .get("sender_id")
      .and_then(|v| v.as_i64())
      .or_else(|| obj.get("senderId").and_then(|v| v.as_i64()))
      .unwrap_or(user_id.0 as i64);

    // Build a flat outbound payload for clients
    let mut outbound_obj = serde_json::Map::new();
    outbound_obj.insert(
      "content".to_string(),
      serde_json::Value::String(content_text.to_string()),
    );
    outbound_obj.insert(
      "room_id".to_string(),
      serde_json::Value::String(room_id_str.to_string()),
    );
    outbound_obj.insert(
      "sender_id".to_string(),
      serde_json::Value::Number(sender_id_val.into()),
    );
    if let Some(idv) = obj.get("id").cloned() {
      outbound_obj.insert("id".to_string(), idv);
    }
    if let Some(ts) = obj
      .get("createdAt")
      .cloned()
      .or_else(|| obj.get("created_at").cloned())
    {
      outbound_obj.insert("createdAt".to_string(), ts);
    } else {
      outbound_obj.insert(
        "createdAt".to_string(),
        serde_json::Value::String(Utc::now().to_rfc3339()),
      );
    }
    let outbound_payload = serde_json::Value::Object(outbound_obj);
    let outbound_payload_str = outbound_payload.to_string();

    // Store only plain text content to DB
    let store_msg = ChatMessageInsertForm {
      room_id: chatroom_id.clone(),
      sender_id: user_id,
      content: content_text.to_string(),
      status: 1,
      created_at: Utc::now(),
      updated_at: None,
    };

    // Serialize once for casting to Phoenix channel & for broker rebroadcast
    let content = outbound_payload_str.clone();

    // Normalize channel from topic ("room:<id>") and map outbound event for clients
    let outbound_channel = ChatRoomId::from_channel_name(&channel_name)
      .unwrap_or_else(|_| ChatRoomId(channel_name.strip_prefix("room:").unwrap_or(&channel_name).to_string()));
    let outbound_event = match event.as_str() {
      "send_message" | "SendMessage" => "chat:message",
      // pass through known page events (history flushes)
      "history_page" => "history_page",
      // default to chat:message for other app events
      _ => "chat:message",
    }
    .to_string();

    tracing::debug!(
      "PHX bridge: inbound_event={}, outbound_event={}, channel_name={}, outbound_channel={}",
      event,
      outbound_event,
      channel_name,
      outbound_channel
    );
    tracing::debug!("PHX bridge: outbound_payload={}", content);

    tracing::debug!(
      "PHX bridge: issue_async -> WebSocket event={} channel={}",
      outbound_event,
      outbound_channel
    );
    if event.eq("typing")
      || event.eq("typing:start")
      || event.eq("typing:stop")
      || event.eq("phx_leave")
    {
      self.issue_async::<SystemBroker, _>(BridgeMessage {
        channel: outbound_channel,
        user_id: msg.user_id.clone(),
        event: outbound_event.clone(),
        messages: content.clone(),
        security_config: false,
      });
      return Box::pin(async move {
        tracing::debug!("PHX bridge: typing event ignored");
      });
    }
    self.issue_async::<SystemBroker, _>(BridgeMessage {
      channel: outbound_channel,
      user_id: msg.user_id.clone(),
      event: outbound_event.clone(),
      messages: content.clone(),
      security_config: false,
    });

    self.add_messages_to_room(chatroom_id, store_msg);
    // Clone mapped event for async move block
    let outbound_event_for_cast = outbound_event.clone();
    Box::pin(async move {
      let arc_chan = get_or_create_channel(channels, socket, &channel_name).await;

      if let Ok(arc_chan) = arc_chan {
        let status = arc_chan.statuses().status().await;
        match status {
          Ok(status) => {
            let phoenix_event = Event::from_string(outbound_event_for_cast.clone());
            let payload: Payload = Payload::binary_from_bytes(content.into_bytes());

            tracing::debug!(
              "PHX cast: event={} status={:?} channel={}",
              outbound_event_for_cast,
              status,
              channel_name
            );

            if status == ChannelStatus::Joined {
              send_event_to_channel(arc_chan, phoenix_event, payload).await;
            } else {
              let _ = arc_chan.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await;
              send_event_to_channel(arc_chan, phoenix_event, payload).await;
            }
          }
          Err(_) => {}
        }
      }
    })
  }
}

#[derive(Message)]
#[rtype(result = "()")]
struct InitSocket(Arc<Socket>);

impl Handler<InitSocket> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: InitSocket, _ctx: &mut Context<Self>) {
    self.socket = msg.0;
    tracing::info!("Connect status: {:?}", self.socket.status());
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
impl Handler<RegisterClientMsg> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: RegisterClientMsg, _ctx: &mut Context<Self>) -> Self::Result {
    let room_id = msg.room_id.clone();
    // let room_name = msg.room_name.clone();
    let user_id = msg.user_id;

    let _ = self.ensure_room_initialized(room_id.clone(), room_id.to_string());

    // Ensure participant exists for this user in this room (create if missing)
    let participant_user_id = user_id;
    if let Some(uid) = participant_user_id {
      let pool_for_participant = self.pool.clone();
      let room_for_participant = room_id.clone();
      let chat_participant_form = ChatParticipantInsertForm {
        room_id: room_for_participant,
        member_id: uid,
      };
      tokio::spawn(async move {
        let mut db_pool = DbPool::Pool(&pool_for_participant);
        let _ = ChatParticipant::ensure_participant(&mut db_pool, &chat_participant_form).await;
      });
    }
  }
}

impl Handler<FetchHistoryDirect> for PhoenixManager {
  type Result = ResponseFuture<FastJobResult<ChatMessagesResponse>>;

  fn handle(&mut self, msg: FetchHistoryDirect, _ctx: &mut Context<Self>) -> Self::Result {
    // Drain buffered messages synchronously while we still have &mut self
    let to_flush = self.drain_room_buffer(&msg.room_id);

    let pool = self.pool.clone();
    let room_id = msg.room_id.clone();
    let user_id = msg.user_id.clone();
    let page_cursor = msg.page_cursor.clone();
    let limit = msg.limit;
    let page_back = msg.page_back;

    Box::pin(async move {
      // Check membership via helper
      ensure_room_membership(pool.clone(), room_id.clone(), user_id.clone()).await?;

      // Persist drained messages, if any, then query
      if !to_flush.is_empty() {
        let mut db_pool = DbPool::Pool(&pool);
        ChatMessage::bulk_insert(&mut db_pool, &to_flush).await?;
      }
      list_chat_messages(pool, room_id, page_cursor, limit, page_back).await
    })
  }
}

// Helper to ensure a user is a member of a room before accessing resources like history
async fn ensure_room_membership(
  pool: ActualDbPool,
  room_id: ChatRoomId,
  user_id: LocalUserId,
) -> FastJobResult<()> {
  let mut db_pool = DbPool::Pool(&pool);
  let participants =
    ChatParticipant::list_participants_for_rooms(&mut db_pool, &[room_id.clone()]).await?;
  let is_member = participants.iter().any(|p| p.member_id == user_id);
  if !is_member {
    return Err(FastJobErrorType::NotAllowed.into());
  }
  Ok(())
}
