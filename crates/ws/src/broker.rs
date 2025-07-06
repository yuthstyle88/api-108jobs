use crate::bridge_message::BridgeMessage;
use crate::message::{RegisterClientKeyMsg, StoreChatMessage};
use actix::{
  Actor, Arbiter, AsyncContext, Context, Handler, Message,
  ResponseFuture,
};
use actix_broker::BrokerSubscribe;
use lemmy_db_schema::newtypes::LocalUserId;
use lemmy_db_schema::source::chat_message::{ChatMessageContent, ChatMessageInsertForm};
use lemmy_db_schema::utils::DbPool;
use lemmy_db_schema::{
  newtypes::ChatRoomId, source::chat_message::ChatMessage, utils::ActualDbPool,
};
use lemmy_utils::error::FastJobResult;
use phoenix_channels_client::{url::Url, Channel, ChannelStatus, Event, Payload, Socket, Topic};
use std::{collections::HashMap, sync::Arc, time::Duration};
use anyhow::Error;
use tokio::sync::RwLock;

#[derive(Message)]
#[rtype(result = "()")]
struct ConnectNow;

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
    let is_valid: Result<bool, _> = async {
      let status = channel.statuses().status().await?;
      if status == ChannelStatus::Joined {
        // Verify with heartbeat
        let test_event = Event::from_string("heartbeat".parse()?);
        let test_payload = Payload::binary_from_bytes(vec![]);
        channel.cast(test_event, test_payload).await?;
        Ok::<bool, Error>(true)
      } else {
        // Try rejoin if not joined
        channel.join(Duration::from_secs(5)).await?;
        Ok(true)
      }
    }.await;

    match is_valid {
      Ok(true) => {
        tracing::debug!("Using existing channel: {}", name);
        return Ok(channel);
      }
      _ => {
        // Remove invalid channel
        let mut write_guard = channels.write().await;
        write_guard.remove(name);
        drop(write_guard); // Explicitly release lock
      }
    }
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

  tracing::debug!("Created new channel: {}", name);
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
  endpoint: String,
  chat_store: HashMap<ChatRoomId, Vec<ChatMessageInsertForm>>,
  pool: ActualDbPool,
  key_cache: KeyCache,
}

impl PhoenixManager {
  pub async fn new(endpoint: &str, pool: ActualDbPool) -> Self {
    let url = Url::parse(endpoint).expect("Invalid endpoint");
    let sock = Socket::spawn(url.clone(), None, None)
      .await
      .expect("Failed to create socket");
    Self {
      socket: sock,
      channels: Arc::new(RwLock::new(HashMap::new())),
      endpoint: endpoint.into(),
      chat_store: HashMap::new(),
      pool,
      key_cache: KeyCache::new(),
    }
  }
}

impl Actor for PhoenixManager {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    ctx.notify(ConnectNow);
    self.subscribe_system_async::<BridgeMessage>(ctx);
    ctx.run_interval(Duration::from_secs(10), |actor, _ctx| {
      let arbiter = Arbiter::new();
      let messages = actor.chat_store.clone();
      let pool = actor.pool.clone();

      let succeeded = arbiter.spawn(async move {

        for (room_id, messages) in messages.iter() {
          if messages.is_empty() {
            continue;
          }

          println!("Flushing {} messages from room {}", messages.len(), room_id);
          let mut db_pool = DbPool::Pool(&pool);
          let _ = ChatMessage::bulk_insert(&mut db_pool, &messages).await;
        }
      });
      if succeeded {
        println!("Task spawned!");
      } else {
        println!("Failed to spawn task.");
      }
      actor.chat_store.clear();
    });
  }
}

// Handler for BridgeMessage
impl Handler<BridgeMessage> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: BridgeMessage, _ctx: &mut Context<Self>) -> Self::Result {
    let channel_name = msg.channel.to_string();
    let user_id = msg.user_id.clone();
    let event = msg.event.clone();
    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);
    let message = msg.messages.clone();

    let client_key = self.key_cache.get(&user_id).unwrap_or_default().to_string();

    let decrypt_data: String;
    if msg.security_config {
      decrypt_data = String::from_utf8(webcryptobox::decrypt(&client_key.as_bytes(), &message.as_bytes()).unwrap()).unwrap().into();
    }else{
      decrypt_data = message.clone().into();
    }
    let content_enum =  ChatMessageContent::from(decrypt_data);
    let chatroom_id = ChatRoomId::from(channel_name.clone());
    let content =  serde_json::to_string(&content_enum).unwrap_or_default();
    //TODO get sender id
    let store_msg = ChatMessageInsertForm{
      room_id: chatroom_id.clone(),
      sender_id: Default::default(),
      content: content.clone(),
      status: 0,
      created_at: Default::default(),
      updated_at: Default::default(),
    };

    self.chat_store.entry(chatroom_id).or_default().push(store_msg);
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

impl Handler<StoreChatMessage> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: StoreChatMessage, _ctx: &mut Context<Self>) -> Self::Result {
    let msg = msg.message;
    self.chat_store.entry(msg.room_id.clone()).or_default().push(msg);
  }
}
impl Handler<RegisterClientKeyMsg> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: RegisterClientKeyMsg, _ctx: &mut Context<Self>) -> Self::Result {
    if msg.user_id.is_some() && msg.client_key.is_some() {
      self.key_cache.insert(msg.user_id.unwrap(), msg.client_key.unwrap());
    }
  }
}
#[derive(Clone)]
pub struct KeyCache {
  map: HashMap<LocalUserId, String>,
}

impl KeyCache {
  pub fn new() -> Self {
    KeyCache {
      map: HashMap::new(),
    }
  }

  // ใส่ key ลง cache (เช่นหลังดึงจาก DB)
  pub fn insert(&mut self, user_id: LocalUserId, key: String) {
    self.map.insert(user_id, key);
  }

  // ดึง key จาก cache
  pub fn get(&self, user_id: &LocalUserId) -> Option<String> {
    self.map.get(user_id).cloned()
  }
}
