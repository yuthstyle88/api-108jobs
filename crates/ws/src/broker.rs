use crate::{
  bridge_message::{BridgeMessage, MessageSource},
  message::{RegisterClientMsg, StoreChatMessage},
};
use actix::{Actor, AsyncContext, Context, Handler, Message, ResponseFuture};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use chrono::Utc;
use lemmy_db_schema::{
  newtypes::{ChatRoomId, LocalUserId},
  source::{
    chat_message::{ChatMessage, ChatMessageInsertForm},
    chat_room::{ChatRoom, ChatRoomInsertForm},
  },
  traits::Crud,
  utils::{ActualDbPool, DbPool},
};
use lemmy_utils::error::FastJobResult;
use phoenix_channels_client::{url::Url, Channel, ChannelStatus, Event, Payload, Socket, Topic};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use lemmy_db_schema::source::chat_participant::{ChatParticipant, ChatParticipantInsertForm};

#[derive(Message)]
#[rtype(result = "()")]
struct ConnectNow;

#[derive(Message)]
#[rtype(result = "()")]
struct FlushDone;

async fn connect(socket: Arc<Socket>) -> FastJobResult<Arc<Socket>> {
  // Try to connect
  match socket.connect(Duration::from_secs(10)).await {
    Ok(_) => Ok(socket),
    Err(e) => {
      eprintln!("Failed to connect to socket: {}", e);
      Err(e.into())
    }
  }
}
async fn send_event_to_channel(channel: Arc<Channel>, event: Event, payload: Payload) {
  if let Err(e) = channel.cast(event, payload).await {
    eprintln!("Failed to cast message: {}", e);
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
        // ถ้าไม่ได้ joined ลอง rejoin
        if let Ok(_) = channel.join(Duration::from_secs(5)).await {
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
    .join(Duration::from_secs(5))
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
impl Handler<ConnectNow> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, _msg: ConnectNow, ctx: &mut Context<Self>) {
    let socket = self.socket.clone();
    let endpoint = self.endpoint.clone();
    let addr = ctx.address();
    tokio::spawn(async move {
      let _fut_connect = match connect(socket).await {
        Ok(sock) => {
          addr.do_send(InitSocket(sock.clone()));
          eprintln!("connect to url: {}", endpoint);
        }
        Err(e) => eprintln!("connected failed: {e}"),
      };
    });
  }
}

pub struct PhoenixManager {
  socket: Arc<Socket>,
  channels: Arc<RwLock<HashMap<String, Arc<Channel>>>>,
  endpoint: Url,
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
      endpoint: endpoint.clone().unwrap(),
      chat_store: HashMap::new(),
      pool,
      is_flushing: false,
    }
  }

  pub async fn validate_or_create_room(
    &mut self,
    room_id: ChatRoomId,
    room_name: String,
  ) -> FastJobResult<()> {
    let room_id = room_id.to_string();
    let mut db_pool = DbPool::Pool(&self.pool);
    if !ChatRoom::exists(&mut db_pool, room_id.clone().into()).await? {
      let now = Utc::now();
      let form = ChatRoomInsertForm {
        id: ChatRoomId(room_id),
        room_name,
        created_at: now,
        updated_at: None,
      };
      ChatRoom::create(&mut db_pool, &form).await?;
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

  fn ensure_room_initialized(&mut self, room_id: ChatRoomId, room_name: String) {
    if !self.chat_store.contains_key(&room_id) {
      // ensure in-memory buffer for this room exists
      self.chat_store.insert(room_id.clone(), Vec::new());

      // asynchronously validate or create the room in DB without blocking the actor
      let pool = self.pool.clone();
      tokio::spawn(async move {
        if let Err(e) = validate_or_create_room_db(pool, room_id, room_name).await {
          eprintln!("Failed to validate/create room in DB: {}", e);
        }
      });
    }
  }
}

impl Actor for PhoenixManager {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    ctx.notify(ConnectNow);
    self.subscribe_system_async::<BridgeMessage>(ctx);
    ctx.run_interval(Duration::from_secs(10), |actor, ctx| {
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
            println!("Failed to flush messages: {}", e);
          }
        }
        addr.do_send(FlushDone);
      });
    });
  }
}

// Handler for BridgeMessage
impl Handler<BridgeMessage> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: BridgeMessage, ctx: &mut Context<Self>) -> Self::Result {
    // Process only messages coming from WebSocket to avoid rebroadcast loops
    if !matches!(msg.source, MessageSource::WebSocket) {
      return Box::pin(async {});
    }

    let channel_name = msg.channel.to_string();
    let user_id = msg.user_id.clone();
    let event = msg.event.clone();

    // Special case: fetch_history should query DB with pagination and emit history, no storage, no phoenix send
    if event == "fetch_history" {
      let pool = self.pool.clone();
      let room_id = ChatRoomId::from_channel_name(channel_name.as_str());
      let addr = ctx.address();

      #[derive(serde::Deserialize)]
      struct Pager { after_id: Option<i32>, limit: Option<i64>, page: Option<i64>, page_size: Option<i64> }
      let parsed = serde_json::from_str::<Pager>(&msg.messages).ok();
      let after_id = parsed.as_ref().and_then(|p| p.after_id);
      let mut limit = parsed.as_ref().and_then(|p| p.limit).unwrap_or(20);
      if limit <= 0 { limit = 20; }
      if limit > 100 { limit = 100; }

      tokio::spawn(async move {
        let mut db_pool = DbPool::Pool(&pool);
        let history = if after_id.is_some() {
          ChatMessage::list_by_room_after_id(&mut db_pool, room_id.clone(), after_id.map(lemmy_db_schema::newtypes::ChatMessageId), limit)
            .await
            .unwrap_or_default()
        } else {
          // Fallback to old paging if clients still send page / page_size
          let (page, page_size) = if let Some(p) = parsed {
            (p.page.unwrap_or(1), p.page_size.unwrap_or(limit))
          } else { (1, limit) };
          ChatMessage::list_by_room_paged(&mut db_pool, room_id.clone(), page, page_size)
            .await
            .unwrap_or_default()
        };
        addr.do_send(HistoryFetched { room_id, user_id: Some(user_id), messages: history });
      });

      return Box::pin(async {});
    }

    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);
    let message = msg.messages.clone();
    let chatroom_id = ChatRoomId::from_channel_name(channel_name.as_str());

    let store_msg = ChatMessageInsertForm {
      room_id: chatroom_id.clone(),
      sender_id: user_id,
      content: message.clone(),
      status: 1,
      created_at: Utc::now(),
      updated_at: None,
    };

    let content = serde_json::to_string(&store_msg).unwrap_or_default();

    // Immediately issue a reply back onto the SystemBroker so connected WsSessions can forward to clients
    self.issue_async::<SystemBroker, _>(BridgeMessage {
      source: MessageSource::Phoenix,
      channel: ChatRoomId::from(channel_name.clone()),
      user_id: msg.user_id.clone(),
      event: event.clone(),
      messages: content.clone(),
      security_config: false,
    });

    self.add_messages_to_room(chatroom_id, store_msg);
    Box::pin(async move {
      let arc_chan = get_or_create_channel(channels, socket, &channel_name).await;

      if let Ok(arc_chan) = arc_chan {
        let status = arc_chan.statuses().status().await;
        match status {
          Ok(status) => {
            let phoenix_event = Event::from_string(event);
            let payload: Payload = Payload::binary_from_bytes(content.into_bytes());

            if status == ChannelStatus::Joined {
              send_event_to_channel(arc_chan, phoenix_event, payload).await;
            } else {
              let _ = arc_chan.join(Duration::from_secs(5)).await;
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
    eprintln!("Connect status : {:?}", self.socket.status());
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

  fn handle(&mut self, msg: RegisterClientMsg, ctx: &mut Context<Self>) -> Self::Result {
    let room_id = msg.room_id.clone();
    let room_name = msg.room_name.clone();
    let user_id = msg.user_id;

    let _ = self.ensure_room_initialized(room_id.clone(), room_name);

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

    // Fetch and emit history asynchronously
    let pool = self.pool.clone();
    let addr = ctx.address();
    tokio::spawn(async move {
      let mut db_pool = DbPool::Pool(&pool);
      let history = ChatMessage::list_by_room(&mut db_pool, room_id.clone(), 10).await.unwrap_or_default();
      addr.do_send(HistoryFetched { room_id, user_id, messages: history });
    });
  }
}

#[derive(Message)]
#[rtype(result = "()")]
struct HistoryFetched {
  room_id: ChatRoomId,
  user_id: Option<LocalUserId>,
  messages: Vec<ChatMessage>,
}

impl Handler<HistoryFetched> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: HistoryFetched, _ctx: &mut Context<Self>) -> Self::Result {
    let channel_name = format!("room:{}", msg.room_id);
    let user_id = msg.user_id.unwrap_or(LocalUserId(0));

    // Emit history as a stream of individual messages (oldest -> newest)
    for m in msg.messages {
      // For compatibility, forward the original stored content field
      // m.content is already a JSON string (e.g., {"Text":{...}}) or plaintext depending on producer.
      // Send the full ChatMessage JSON instead of only the content string
      let full_msg_json = match serde_json::to_string(&m) {
        Ok(s) => s,
        Err(_) => {
          // Fallback to original content in case of serialization error
          m.content.clone()
        }
      };
      self.issue_async::<SystemBroker, _>(BridgeMessage {
        source: MessageSource::Phoenix,
        channel: ChatRoomId::from(channel_name.clone()),
        user_id,
        event: "history_item".to_string(),
        messages: full_msg_json,
        security_config: false,
      });
    }

    // Do not emit an extra history_end message; clients should infer end by stream completion
    // (Removed previously sent empty/status payload to avoid confusing consumers)
  }
}

// Helper function to validate or create a chat room in the DB without borrowing PhoenixManager
async fn validate_or_create_room_db(
  pool: ActualDbPool,
  room_id: ChatRoomId,
  room_name: String,
) -> FastJobResult<()> {
  let room_id_str = room_id.to_string();
  let mut db_pool = DbPool::Pool(&pool);

  if !ChatRoom::exists(&mut db_pool, room_id_str.clone().into()).await? {
    let now = Utc::now();
    let form = ChatRoomInsertForm {
      id: ChatRoomId(room_id_str.clone()),
      room_name,
      created_at: now,
      updated_at: None,
    };
    ChatRoom::create(&mut db_pool, &form).await?;
  }

  Ok(())
}
