use crate::{
  bridge_message::{BridgeMessage, MessageSource},
  broker::PhoenixManager,
  message::RegisterClientMsg,
};
use actix::{Actor, Addr, Handler, StreamHandler};
use actix::ActorContext;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use serde::Deserialize;

pub struct WsSession {
  pub(crate) phoenix_manager: Addr<PhoenixManager>,
  pub(crate) client_msg: RegisterClientMsg,
}

impl WsSession {
  pub fn new(
    phoenix_manager: Addr<PhoenixManager>,
    client_msg: RegisterClientMsg,
  ) -> Self {
    Self {
      phoenix_manager,
      client_msg,
    }
  }

  fn maybe_encrypt_outbound<'a>(&'a self, _event: &str, messages: &'a str) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    // Pass-through: deliver messages as-is; clients handle any encryption.
    Cow::Borrowed(messages)
  }

  fn maybe_decrypt_incoming(&self, _content: &str) -> Option<String> {
    // Do not attempt to decrypt on the server. Treat incoming as plaintext.
    None
  }
}
impl Actor for WsSession{
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.subscribe_system_sync::<BridgeMessage>(ctx);
    let user_id = self.client_msg.user_id;
    let room_id = self.client_msg.room_id.clone();
    let room_name = self.client_msg.room_name.clone();
    self.phoenix_manager.do_send(RegisterClientMsg { user_id, room_id, room_name });
  }
}

impl Handler<BridgeMessage> for WsSession {
  type Result = ();

  fn handle(&mut self, msg: BridgeMessage, ctx: &mut Self::Context) {
    // Only forward messages that originate from Phoenix to the client, to avoid echo/loops
    if !matches!(msg.source, MessageSource::Phoenix) {
      return;
    }

    // Deliver only messages for this session's room
    if ChatRoomId::from_channel_name(msg.channel.as_ref()) != self.client_msg.room_id {
      return;
    }

    let outbound = self.maybe_encrypt_outbound(&msg.event, &msg.messages);
    ctx.text(outbound.as_ref());
  }
}
#[derive(Deserialize, Debug)]
pub enum MessageOp {
  SendMessage,
  LeaveRoom,
  JoinRoom,
}

impl std::fmt::Display for MessageOp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MessageOp::SendMessage => write!(f, "send_message"),
      MessageOp::LeaveRoom => write!(f, "leave_room"),
      MessageOp::JoinRoom => write!(f, "join_room"),
    }
  }
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MessageRequest {
  pub op: MessageOp,
  pub sender_id: LocalUserId,
  pub room_id: ChatRoomId,
  pub content: String,
}


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Text(text)) => {
        tracing::info!("Received: {}", text);

        // First, try to parse as the original backend format
        if let Ok(value) = serde_json::from_str::<MessageRequest>(&text) {
          let messages = self
            .maybe_decrypt_incoming(&value.content)
            .unwrap_or_else(|| value.content.clone());

          let bridge_msg = BridgeMessage {
            source: MessageSource::WebSocket,
            channel: format!("room:{}", value.room_id).into(),
            user_id: value.sender_id,
            event: value.op.to_string(),
            messages,
            security_config: false,
          };
          self.issue_async::<SystemBroker, _>(bridge_msg);
        } else {
          tracing::warn!("Failed to parse incoming message as known formats");
        }
      }

      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      Ok(ws::Message::Close(_)) => ctx.stop(),

      Err(err) => {
        tracing::error!("WebSocket protocol error: {:?}", err);
        ctx.stop();
      }

      _ => {}
    }
  }
}
