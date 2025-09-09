use crate::{
  bridge_message::{BridgeMessage, MessageSource},
  broker::PhoenixManager,
  message::RegisterClientMsg,
};
use actix::{Actor, Addr, Handler, StreamHandler};
use actix::ActorContext;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId, PaginationCursor};
use lemmy_utils::crypto::{xchange_decrypt_data, xchange_encrypt_data};
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
      shared_key,
    }
  }

  #[inline]
  fn has_security(&self) -> bool {
    !self.shared_key.is_empty() && !self.session_id.is_empty()
  }

  fn encrypt_content_fields(
    &self,
    value: &mut serde_json::Value,
  ) -> bool {
    let mut changed = false;

    // Try nested message.content first
    if let Some(content_val) = value
      .get_mut("message")
      .and_then(|m| m.get_mut("content"))
    {
      if let Some(content_str) = content_val.as_str() {
        match xchange_encrypt_data(content_str, &self.shared_key, &self.session_id) {
          Ok(enc) => {
            *content_val = serde_json::Value::String(enc);
            changed = true;
          }
          Err(err) => {
            tracing::error!("Encrypt content (message.content) failed: {:?}", err);
          }
        }
      }
    }

    // Fallback to root-level content when nested not found/changed
    if !changed {
      if let Some(root_content_val) = value.get_mut("content") {
        if let Some(content_str) = root_content_val.as_str() {
          match xchange_encrypt_data(content_str, &self.shared_key, &self.session_id) {
            Ok(enc) => {
              *root_content_val = serde_json::Value::String(enc);
              changed = true;
            }
            Err(err) => {
              tracing::error!("Encrypt content (root content) failed: {:?}", err);
            }
          }
        }
      }
    }

    changed
  }

  fn maybe_encrypt_outbound<'a>(&'a self, event: &str, messages: &'a str) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;

    if !(event == "history_item" || event == "send_message") || !self.has_security() {
      return Cow::Borrowed(messages);
    }

    match serde_json::from_str::<serde_json::Value>(messages) {
      Ok(mut value) => {
        if self.encrypt_content_fields(&mut value) {
          match serde_json::to_string(&value) {
            Ok(s) => Cow::Owned(s),
            Err(_) => Cow::Borrowed(messages),
          }
        } else {
          Cow::Borrowed(messages)
        }
      }
      Err(_) => Cow::Borrowed(messages),
    }
  }

  fn maybe_decrypt_incoming(&self, content: &str) -> Option<String> {
    if !self.has_security() {
      return None;
    }
    match xchange_decrypt_data(content, &self.shared_key, &self.session_id) {
      Ok(messages) => Some(messages),
      Err(err) => {
        tracing::warn!("Decryption error: {:?}. Falling back to plaintext content.", err);
        None
      }
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

    let outbound = self.maybe_encrypt_outbound(&msg.event, &msg.messages);
    ctx.text(outbound.as_ref());
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
  pub room_id: ChatRoomId,
  pub content: String,
  // New cursor-based pagination (preferred)
  #[serde(default)]
  pub page_cursor: Option<PaginationCursor>,
  #[serde(default)]
  pub page_back: Option<bool>,
  #[serde(default)]
  pub limit: Option<i64>,
}


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Text(text)) => {
        tracing::debug!("Received: {}", text);

        // First, try to parse as the original backend format
        if let Ok(value) = serde_json::from_str::<MessageRequest>(&text) {
          // For fetch_history, forward only cursor pagination parameters
          let messages = if matches!(value.op, MessageOp::FetchHistory) {
            #[derive(serde::Serialize)]
            struct Pager {
              page_cursor: Option<PaginationCursor>,
              page_back: Option<bool>,
              limit: Option<i64>,
            }
            serde_json::to_string(&Pager {
              page_cursor: value.page_cursor.clone(),
              page_back: value.page_back,
              limit: value.limit,
            })
            .unwrap_or_else(|_| "{}".to_string())
          } else {
            self.maybe_decrypt_incoming(&value.content)
              .unwrap_or_else(|| value.content.clone())
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

