use crate::{
  bridge_message::{BridgeMessage, MessageSource},
  broker::PhoenixManager,
  message::RegisterClientMsg,
};
use actix::{Actor, ActorContext, Addr, Handler, StreamHandler};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use serde::Deserialize;

pub struct WsSession {
  pub(crate) phoenix_manager: Addr<PhoenixManager>,
  pub(crate) client_msg: RegisterClientMsg,
}

impl Actor for WsSession {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.subscribe_system_sync::<BridgeMessage>(ctx);
    let user_id = self.client_msg.user_id;
    let client_key = self.client_msg.client_key.clone();
    let room_id = self.client_msg.room_id.clone();
    let room_name = self.client_msg.room_name.clone();
    self.phoenix_manager.do_send(RegisterClientMsg { user_id,  client_key, room_id, room_name });
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
pub enum MessageOp {
  SendMessage,
  LeaveRoom,
  JoinRoom,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MessageRequest {
  pub op: MessageOp,
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
            let bridge_msg = BridgeMessage {
              source: MessageSource::WebSocket,
              channel: format!("room:{}", value.room_id).into(),
              user_id: value.sender_id,
              event: value.op.to_string(),
              messages: value.content,
              security_config: false,
            };
            self.issue_async::<SystemBroker, _>(bridge_msg);
          }
        },
      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      Ok(ws::Message::Close(_)) => ctx.stop(),
      _ => {}
    }
  }
}
