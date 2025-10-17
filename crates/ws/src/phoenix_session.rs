use crate::bridge_message::OutboundMessage;
use crate::broker::helper::{is_base64_like, parse_phx, phx_push, phx_reply};
use crate::impls::AnyIncomingEvent;
use crate::bridge_message::BridgeMessage;
use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use lemmy_utils::crypto;
use serde_json::Value;
use std::borrow::Cow;
use crate::api::ChatEvent;

// ===== actor =====
pub struct PhoenixSession {
  pub(crate) shared_key: Option<String>,
}

impl PhoenixSession {
  pub fn new(
    shared_key: Option<String>,
  ) -> Self {
    Self {
      shared_key,
    }
  }

  /// Encrypt a plaintext string with AES-GCM (shared key is stored as hex -> 32 bytes),
  /// returning base64-encoded ciphertext. If no key / invalid key, return the original text.
  fn maybe_encrypt_outbound<'a>(&'a self, plaintext: &'a str) -> Cow<'a, str> {
    let Some(shared_hex) = self.shared_key.as_ref() else {
      return Cow::Borrowed(plaintext);
    };
    let Ok(key_bytes) = hex::decode(shared_hex) else {
      return Cow::Borrowed(plaintext);
    };
    if key_bytes.len() != 32 {
      return Cow::Borrowed(plaintext);
    }
    match crypto::encrypt_string_b64(&key_bytes, plaintext) {
      Ok(s) => Cow::Owned(s),
      Err(_) => Cow::Borrowed(plaintext),
    }
  }

  /// Try to decrypt a base64-encoded AES-GCM payload using the shared key (hex -> 32 bytes).
  fn maybe_decrypt_incoming<'a>(&'a self, messages: &'a str) -> Cow<'a, str> {
    let Some(shared_hex) = self.shared_key.as_ref() else {
      return Cow::Borrowed(messages);
    };
    if !is_base64_like(messages) {
      return Cow::Borrowed(messages);
    }
    let Ok(key_bytes) = hex::decode(shared_hex) else {
      return Cow::Borrowed(messages);
    };
    if key_bytes.len() != 32 {
      return Cow::Borrowed(messages);
    }
    match crypto::decrypt_string_b64(&key_bytes, messages) {
      Ok(s) => Cow::Owned(s),
      Err(_) => Cow::Borrowed(messages),
    }
  }
}

impl Actor for PhoenixSession {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.subscribe_system_sync::<OutboundMessage>(ctx);
  }
}

impl Handler<OutboundMessage> for PhoenixSession {
  type Result = ();

  fn handle(&mut self, msg: OutboundMessage, ctx: &mut Self::Context) {
    let mut payload_val = msg.out_event.payload;
    let topic = msg.out_event.topic.clone();
    let event = msg.out_event.event.clone();

    // Encrypt only when the payload marks it as secure
    let secure = payload_val.get("secure").and_then(|v| v.as_bool()).unwrap_or(false);
    if secure && matches!(event, ChatEvent::Message) {
      if let Some(content_val) = payload_val.get_mut("content") {
        // Support both string and JSON content by normalizing to a plaintext string first
        let plaintext: String = match content_val {
          serde_json::Value::String(s) => s.clone(),
          ref other => serde_json::to_string(other).unwrap_or_default(),
        };
        let encrypted = self.maybe_encrypt_outbound(&plaintext);
        *content_val = serde_json::Value::String(encrypted.into_owned());
      }
    }
    // Build the Phoenix push frame from the topic/event/payload as before
    let frame = phx_push(&topic, &event, payload_val);
    ctx.text(frame.as_ref());
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PhoenixSession {
  fn handle(&mut self, m: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match m {
      Ok(ws::Message::Text(txt)) => {
        if let Some((jr, mr, mut incoming)) = parse_phx(&txt) {
          // Echo back exactly what the client sent (no decryption/encryption for reply)
          let reply: Value = incoming.payload.clone();
          // 1) Decrypt in-place on the raw incoming payload (so reply & broker both see plaintext)
          if let Some(obj) = incoming.payload.as_object_mut() {
            let secure = obj.get("secure").and_then(|v| v.as_bool()).unwrap_or(false);
            if secure {
              if let Some(content_val) = obj.get_mut("content") {
                if let Some(content_str) = content_val.as_str() {
                  let decrypted = self.maybe_decrypt_incoming(content_str);
                  *content_val = serde_json::Value::String(decrypted.into_owned());
                }
              }
            }
          }

          let any_event: AnyIncomingEvent = AnyIncomingEvent::from(incoming.clone());
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
