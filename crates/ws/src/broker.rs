use actix::{Actor, Arbiter, AsyncContext, Context, Handler, Message, ResponseFuture};
use actix_broker::BrokerSubscribe;
use phoenix_channels_client::url::Url;
use phoenix_channels_client::{Channel, Event, Payload, Socket, Topic};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use lemmy_db_schema::utils::ActualDbPool;
use lemmy_utils::error::FastJobResult;
use crate::bridge_message::BridgeMessage;

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
async fn send_event_to_channel(
    channel: &Arc<Channel>,
    event: Event,
    payload: Value,
) {
    match Payload::json_from_serialized(payload.to_string()) {
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
            pool,
        }
    }
}

impl Actor for PhoenixManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.notify(ConnectNow);
        self.subscribe_system_async::<BridgeMessage>(ctx);
        ctx.run_interval(Duration::from_secs(10), |_actor, _ctx| {
            let arbiter = Arbiter::new();
            let succeeded = arbiter.spawn(async {
                println!("interval task async job done!");
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
        let channel_name = msg.channel.clone();
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

            // ส่ง event ตามเดิม
            let phoenix_event = Event::from_string(event);
            send_event_to_channel(&arc_chan, phoenix_event, msg.payload).await;
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
