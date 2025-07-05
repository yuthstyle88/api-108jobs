use crate::bridge_message::BridgeMessage;
use actix::{fut::wrap_future, Actor, Arbiter, AsyncContext, Context, Handler, Message, MessageResult, ResponseFuture};
use actix_broker::BrokerSubscribe;
use lemmy_db_schema::{
  newtypes::ChatRoomId,
  source::chat_message::ChatMessage,
  utils::ActualDbPool,
};
use lemmy_utils::error::FastJobResult;
use phoenix_channels_client::{url::Url, Channel, Event, Payload, Socket, Topic};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use lemmy_db_schema::utils::DbPool;
use crate::message::StoreChatMessage;

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
async fn send_event_to_channel(channel: &Arc<Channel>, event: Event, payload: String) {
  match Payload::json_from_serialized(payload) {
    Ok(payload) => {
      if let Err(e) = channel.cast(event, payload).await {
        eprintln!("Failed to cast message: {}", e);
      }
    }
    Err(e) => {
      eprintln!("Failed to serialize payload: {}", e);
    }
  }
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
  channels: HashMap<String, Arc<Channel>>,
  endpoint: String,
  chat_store: HashMap<ChatRoomId, Vec<ChatMessageInsertForm>>,
  pool: ActualDbPool,
}

impl PhoenixManager {
  pub async fn new(endpoint: &str, pool: ActualDbPool) -> Self {
    let url = Url::parse(endpoint).expect("Invalid endpoint");
    let sock = Socket::spawn(url.clone(), None, None)
      .await
      .expect("Failed to create socket");
    Self {
      socket: sock,
      channels: HashMap::new(),
      endpoint: endpoint.into(),
      chat_store: HashMap::new(),
      pool,
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
    let event = msg.event.clone();
    let socket = self.socket.clone();
    let mut channels = self.channels.clone();

    Box::pin(async move {
      let arc_chan = if let Some(chan) = channels.get(&channel_name).cloned() {
        chan
      } else {
        let channel = Topic::from_string(channel_name.clone());
        match socket.channel(channel, None).await {
          Ok(new_chan) => {
            let arc_chan = new_chan;
            channels.insert(channel_name.clone(), arc_chan.clone());
            arc_chan
          }
          Err(e) => {
            eprintln!("Failed to create channel '{}': {}", channel_name, e);
            return;
          }
        }
      };

      if let Err(e) = arc_chan.join(Duration::from_secs(5)).await {
        eprintln!("Failed to join channel '{}': {}", channel_name, e);
        return;
      }

      let phoenix_event = Event::from_string(event);
      send_event_to_channel(&arc_chan, phoenix_event, msg.messages).await;
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

    fn handle(&mut self, msg: StoreChatMessage, ctx: &mut Context<Self>) -> Self::Result {
        let mut store = self.chat_store.clone();
        let msg = msg.message;

        let fut = async move {

          store.entry(msg.room_id).or_default().push(msg);
        };

        ctx.spawn(wrap_future(fut));
    }
}
