use crate::{
  bridge_message::{BridgeMessage, MessageSource},
  broker::PhoenixManager,
  message::RegisterClientMsg,
};
use actix::{Actor, ActorContext, Addr, Handler, StreamHandler};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use lemmy_utils::crypto::xchange_decrypt_data;
use serde::Deserialize;

pub struct WsSession {
  pub(crate) phoenix_manager: Addr<PhoenixManager>,
  pub(crate) client_msg: RegisterClientMsg,
  pub(crate) session_id: String,
  pub(crate) shared_key: String,
}

impl WsSession {
  pub fn new(
    phoenix_manager: Addr<PhoenixManager>,
    client_msg: RegisterClientMsg,
    session_id: String,
    shared_key: String,
  ) -> Self {
    Self {
      phoenix_manager,
      client_msg,
      session_id,
      shared_key
    }
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

    // Send plaintext list/data to client without encryption, per requirement
    ctx.text(msg.messages);
  }
}
#[derive(Deserialize, Debug)]
pub enum MessageOp {
  SendMessage,
  LeaveRoom,
  JoinRoom,
  FetchHistory,
}

impl std::fmt::Display for MessageOp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MessageOp::SendMessage => write!(f, "send_message"),
      MessageOp::LeaveRoom => write!(f, "leave_room"),
      MessageOp::JoinRoom => write!(f, "join_room"),
      MessageOp::FetchHistory => write!(f, "fetch_history"),
    }
  }
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MessageRequest {
  pub op: MessageOp,
  pub sender_id: LocalUserId,
  pub receiver_id: LocalUserId,
  pub room_id: ChatRoomId,
  pub content: String,
  #[serde(default)]
  pub page: Option<i64>,
  #[serde(default)]
  pub page_size: Option<i64>,
}


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Text(text)) => {
        println!("Received: {}", text);

        // First, try to parse as the original backend format
        if let Ok(value) = serde_json::from_str::<MessageRequest>(&text) {
          // For fetch_history, forward page parameters instead of content
          let messages = if matches!(value.op, MessageOp::FetchHistory) {
            #[derive(serde::Serialize)]
            struct Pager { page: Option<i64>, page_size: Option<i64> }
            serde_json::to_string(&Pager { page: value.page, page_size: value.page_size })
              .unwrap_or_else(|_| "{\"page\":null,\"page_size\":null}".to_string())
          } else {
            value.content.clone()
          };

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
          eprintln!("Failed to parse incoming message as known formats");
        }
      }

      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      Ok(ws::Message::Close(_)) => ctx.stop(),

      Err(err) => {
        eprintln!("WebSocket protocol error: {:?}", err);
        ctx.stop();
      }

      _ => {}
    }
  }
}

