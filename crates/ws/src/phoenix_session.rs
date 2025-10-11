use crate::api::{ChatEvent, MessageModel};
use crate::bridge_message::OutboundMessage;
use crate::broker::helper::{is_base64_like, parse_phx, phx_push, phx_reply};
use crate::broker::phoenix_manager::PhoenixManager;
use crate::broker::presence_manager::{OnlineJoin, OnlineLeave, PresenceManager};
use crate::impls::AnyIncomingEvent;
use crate::{api::RegisterClientMsg, bridge_message::BridgeMessage};
use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use chrono::Utc;
use lemmy_utils::crypto;
use serde_json::Value;
use std::borrow::Cow;
use lemmy_db_views_chat::api::JoinRoomQuery;

// ===== actor =====
pub struct PhoenixSession {
  pub(crate) phoenix_manager: Addr<PhoenixManager>,
  pub(crate) presence_manager: Addr<PresenceManager>,
  #[allow(dead_code)]
  pub(crate) params: JoinRoomQuery,
  pub(crate) client_msg: RegisterClientMsg,
  pub(crate) secure: bool,
  pub(crate) shared_key_hex: Option<String>,
  pub(crate) session_id: Option<String>,
}

impl PhoenixSession {
  pub fn new(
    phoenix_manager: Addr<PhoenixManager>,
    presence_manager: Addr<PresenceManager>,
    params: JoinRoomQuery,
    client_msg: RegisterClientMsg,
  ) -> Self {
    Self {
      phoenix_manager,
      presence_manager,
      params,
      client_msg,
      secure: true,
      shared_key_hex: None,
      session_id: None,
    }
  }

  /// Try to encrypt only the payload.content field (if present and plaintext).
  fn maybe_encrypt_outbound<'a>(&'a self, messages: &'a str) -> Cow<'a, str> {
    let Some(shared) = self.secure.then(|| self.shared_key_hex.as_ref()).flatten() else {
      return Cow::Borrowed(messages);
    };

    // Avoid attempting decrypt on plaintext
    if !is_base64_like(messages) {
      return Cow::Borrowed(messages);
    }

    let session_id = self.session_id.as_deref().unwrap_or("");
    match crypto::xchange_encrypt_data(messages, shared, session_id) {
      Ok(s) => Cow::Owned(s),
      Err(_) => Cow::Borrowed(messages),
    }
  }

  /// Try to decrypt only when:
  ///  - secure mode is on
  ///  - shared key exists
  ///  - payload looks like base64-like ciphertext
  /// Returns borrowed `messages` when not applicable or decryption fails.
  fn maybe_decrypt_incoming<'a>(&'a self, messages: &'a str) -> Cow<'a, str> {
    // Fast path: not in secure mode or no key → return as-is
    let Some(shared) = self.secure.then(|| self.shared_key_hex.as_ref()).flatten() else {
      return Cow::Borrowed(messages);
    };

    // Avoid attempting decrypt on plaintext
    if !is_base64_like(messages) {
      return Cow::Borrowed(messages);
    }

    let session_id = self.session_id.as_deref().unwrap_or("");
    match crypto::xchange_decrypt_data(messages, shared, session_id) {
      Ok(s) => Cow::Owned(s),
      Err(_) => Cow::Borrowed(messages),
    }
  }
}

impl Actor for PhoenixSession {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.subscribe_system_sync::<OutboundMessage>(ctx);
    // Register this client/room with the manager (similar to WsSession)
    let local_user_id = self.client_msg.local_user_id;
    let room_id = self.client_msg.room_id.clone();
    self.phoenix_manager.do_send(RegisterClientMsg {
      local_user_id,
      room_id: room_id.clone(),
    });
    // Notify presence directly (method #1): only if we know user_id
    if let Some(uid) = local_user_id {
      self.presence_manager.do_send(OnlineJoin {
        local_user_id: uid.0,
        started_at: Utc::now(),
      });
    }
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
    let local_user_id = self.client_msg.local_user_id;

    if let Some(uid) = local_user_id {
      self.presence_manager.do_send(OnlineLeave {
        local_user_id: uid.0,
        left_at: Utc::now(),
      });
    }
  }
}

impl Handler<OutboundMessage> for PhoenixSession {
  type Result = ();

  fn handle(&mut self, msg: OutboundMessage, ctx: &mut Self::Context) {
    let payload_val = msg.out_event.payload;
    let topic = msg.out_event.topic.clone();
    let event = msg.out_event.event.clone();

    let frame = phx_push(&topic, &event, payload_val);

    let output = match event {
      ChatEvent::Message => self.maybe_encrypt_outbound(&frame),
      _ => Cow::Borrowed(frame.as_str()),
    };

    ctx.text(output.as_ref());
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PhoenixSession {
  fn handle(&mut self, m: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match m {
      Ok(ws::Message::Text(txt)) => {
        if let Some((jr, mr, incoming)) = parse_phx(&txt) {
          let any_event: AnyIncomingEvent = AnyIncomingEvent::from(incoming.clone());
          let payload_opt = match any_event.clone() {
            AnyIncomingEvent::Message(mut ev) => {
              // Take payload out to avoid double-unwrap and mutate safely
              if let Some(mut p) = ev.payload.take() {
                if let Some(ref mut content) = p.content {
                    let decrypted = self.maybe_decrypt_incoming(content);
                    *content = decrypted.into_owned();
                }
                Some(p)
              } else {
                None
              }
            }
            AnyIncomingEvent::Read(ev) => ev.payload.map(MessageModel::from),
            _ => None,
          };

          let reply = match payload_opt {
            Some(p) => serde_json::to_value(p).unwrap_or(Value::Null),
            None => serde_json::json!({}),
          };

          let bridge_msg = BridgeMessage {
            any_event,
            incoming_event: incoming.clone(),
          };
          self.issue_async::<SystemBroker, _>(bridge_msg);
          ctx.text(phx_reply(&jr, &mr, &incoming.topic, "ok", reply));
        }
      }
      Ok(ws::Message::Ping(b)) => ctx.pong(&b),
      Ok(ws::Message::Close(r)) => ctx.close(r),
      _ => {}
    }
  }
}
