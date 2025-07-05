use crate::bridge_message::BridgeMessage;
use crate::message::StoreChatMessage;
use actix::{
  Actor, Arbiter, AsyncContext, Context, Handler, Message,
  ResponseFuture,
};
use actix_broker::BrokerSubscribe;
use lemmy_db_schema::newtypes::{ClientKey, LocalUserId};
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use lemmy_db_schema::utils::DbPool;
use lemmy_db_schema::{
  newtypes::ChatRoomId, source::chat_message::ChatMessage, utils::ActualDbPool,
};
use lemmy_utils::error::FastJobResult;
use phoenix_channels_client::{
  url::Url, Channel, ChannelStatus, Event, Payload, Socket, Topic,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{Mutex, RwLock};

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
  channels: Arc<Mutex<HashMap<String, Arc<Channel>>>>,
  socket: Arc<Socket>,
  name: &str,
) -> FastJobResult<Arc<Channel>> {
  let mut channels = channels.lock().await;
  if !channels.contains_key(name) {
    let topic = Topic::from_string(name.to_string());
    let chan = socket.channel(topic, None).await?;
    channels.insert(name.to_string(), chan.clone());
  }
  Ok(Arc::clone(channels.get(name).unwrap()))
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
  channels: Arc<Mutex<HashMap<String, Arc<Channel>>>>,
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
      channels: Arc::new(Mutex::new(HashMap::new())),
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
      let mut store = actor.chat_store.clone();
      let pool = actor.pool.clone();

      let succeeded = arbiter.spawn(async move {
        // let mut map = store.write().await;

        for (room_id, messages) in store.drain() {
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
    let key_cache = self.key_cache.clone();
    Box::pin(async move {
      let client_key = key_cache
        .get(&user_id)
        .await
        .unwrap()
        .0
        .public_key_to_der()
        .unwrap();
      let arc_chan = get_or_create_channel(channels, socket, &channel_name).await;

      if let Ok(arc_chan) = arc_chan {
        let status = arc_chan.statuses().status().await;
        match status {
          Ok(status) => {
            let phoenix_event = Event::from_string(event);
            let decrypt_date = webcryptobox::decrypt(&client_key, &message.as_bytes());
            let payload: Payload = Payload::binary_from_bytes(decrypt_date.unwrap());
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
    self.chat_store.entry(msg.room_id).or_default().push(msg);
  }
}
#[derive(Clone)]
pub struct KeyCache {
  map: Arc<RwLock<HashMap<LocalUserId, ClientKey>>>,
}

impl KeyCache {
  pub fn new() -> Self {
    KeyCache {
      map: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  // ใส่ key ลง cache (เช่นหลังดึงจาก DB)
  pub async fn insert(&self, user_id: LocalUserId, key: ClientKey) {
    let mut w = self.map.write().await;
    w.insert(user_id, key);
  }

  // ดึง key จาก cache
  pub async fn get(&self, user_id: &LocalUserId) -> Option<ClientKey> {
    let r = self.map.read().await;
    r.get(user_id).cloned()
  }
}
