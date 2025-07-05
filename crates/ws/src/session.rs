use std::error::Error;
use crate::{
  bridge_message::{BridgeMessage, MessageSource},
  broker::PhoenixManager,
  crypto::Crypto
};
use actix::{Actor, ActorContext, Addr, Handler, StreamHandler};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use lemmy_db_schema::newtypes::PostId;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use serde::Deserialize;

pub struct WsSession {
  pub(crate) crypto: Crypto,
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
    if let Ok(text) = serde_json::to_string(&msg.messages) {
      ctx.text(text);
    }
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MessageRequest {
  pub op: String, // เช่น "send_message", "leave_room", ...
  pub sender_id: LocalUserId,
  pub receiver_id: LocalUserId,
  pub room_id: ChatRoomId,
  pub content: String,
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Text(text)) => {
        println!("Received: {}", text);
        if let Ok(value) = serde_json::from_str::<MessageRequest>(&text) {
          let content = self.crypto.encrypt(&value.content);
          if let Some(data) = content {
            let bridge_msg = BridgeMessage {
              op,
              source: MessageSource::WebSocket,
              channel: value.room_id.into(), // Change this based on your needs
              event: "new_msg".to_string(),      // Change this based on your needs
              messages: data,
            };
            self.issue_async::<SystemBroker, _>(bridge_msg);
            }
          }
        },
      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      Ok(ws::Message::Close(_)) => ctx.stop(),
      _ => {}
    }
  }
}
