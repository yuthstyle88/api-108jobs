use actix::{Actor, ActorContext, Addr, Handler, StreamHandler};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use serde_json::Value;
use crate::bridge_message::{BridgeMessage, MessageSource};
use crate::broker::PhoenixManager;

pub struct WsSession {
    pub(crate) id: String,
    pub(crate) phoenix_manager: Addr<PhoenixManager>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_sync::<BridgeMessage>(ctx);
    }
}

impl Handler<BridgeMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: BridgeMessage, ctx: &mut Self::Context) {
        // Handle messages from broker
        if let Ok(text) = serde_json::to_string(&msg.payload) {
            ctx.text(text);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Received: {}", text);
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    let bridge_msg = BridgeMessage {
                        source: MessageSource::WebSocket,
                        channel: "room:lobby".to_string(), // Change this based on your needs
                        event: "new_msg".to_string(),      // Change this based on your needs
                        payload: value,
                    };
                    self.issue_async::<SystemBroker, _>(bridge_msg);
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => ctx.stop(),
            _ => {}
        }
    }
}
