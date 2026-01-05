use actix::{Actor, Handler, StreamHandler};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use chrono::Utc;
use serde_json::Value;
use crate::bridge_message::{BridgeMessage, GlobalOffline, GlobalOnline, OutboundMessage};
use crate::protocol::phx_helper::{is_base64_like, parse_phx, phx_push, phx_reply};
use crate::protocol::impls::AnyIncomingEvent;
use crate::protocol::api::{ChatEvent, IncomingEvent};
use uuid::Uuid;
use app_108jobs_db_schema::newtypes::LocalUserId;
use app_108jobs_utils::crypto;

// ===== actor =====
pub struct PhoenixSession {
  pub(crate) shared_key: Option<String>,
  pub(crate) local_user_id: Option<LocalUserId>,
  pub(crate) connection_id: String,
}

impl PhoenixSession {
  pub fn new(
    shared_key: Option<String>,
    local_user_id: Option<LocalUserId>,
  ) -> Self {
    Self {
      shared_key,
      local_user_id,
      connection_id: Uuid::new_v4().to_string(),
    }
  }

  /// Return a 32-byte key decoded from shared_key hex, or None if absent/invalid.
  fn key_bytes(&self) -> Option<Vec<u8>> {
    let hex_key = self.shared_key.as_ref()?;
    let bytes = hex::decode(hex_key).ok()?;
    if bytes.len() == 32 { Some(bytes) } else { None }
  }

  #[inline]
  fn encrypt_str(&self, plaintext: &str) -> Option<String> {
    let key = self.key_bytes()?;
    crypto::encrypt_string_b64(&key, plaintext).ok()
  }

  #[inline]
  fn decrypt_str(&self, b64: &str) -> Option<String> {
    if !is_base64_like(b64) { return None; }
    let key = self.key_bytes()?;
    crypto::decrypt_string_b64(&key, b64).ok()
  }

  #[inline]
  fn is_secure(payload: &Value) -> bool {
    payload.get("secure").and_then(|v| v.as_bool()).unwrap_or(false)
  }
}

impl Actor for PhoenixSession {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.subscribe_system_sync::<OutboundMessage>(ctx);

    // Emit GlobalOnline if user is authenticated
    if let Some(uid) = self.local_user_id {
        let ev = GlobalOnline {
            local_user_id: uid,
            connection_id: self.connection_id.clone(),
            at: Utc::now(),
        };
        let bridge_msg = BridgeMessage {
            any_event: AnyIncomingEvent::GlobalOnline(ev),
            incoming_event: IncomingEvent {
                event: ChatEvent::Unknown, // Placeholder
                room_id: app_108jobs_db_schema::newtypes::ChatRoomId("global".to_string()),
                topic: "global".to_string(),
                payload: Value::Null,
            }
        };
        self.issue_async::<SystemBroker, _>(bridge_msg);
    }
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
      // Emit GlobalOffline if user is authenticated
      if let Some(uid) = self.local_user_id {
          let ev = GlobalOffline {
              local_user_id: uid,
              connection_id: self.connection_id.clone(),
          };
          let bridge_msg = BridgeMessage {
              any_event: AnyIncomingEvent::GlobalOffline(ev),
              incoming_event: IncomingEvent {
                  event: ChatEvent::Unknown, // Placeholder
                  room_id: app_108jobs_db_schema::newtypes::ChatRoomId("global".to_string()),
                  topic: "global".to_string(),
                  payload: serde_json::Value::Null,
              }
          };
          self.issue_async::<SystemBroker, _>(bridge_msg);
      }
  }
}

impl Handler<OutboundMessage> for PhoenixSession {
  type Result = ();

  fn handle(&mut self, msg: OutboundMessage, ctx: &mut Self::Context) {
    let mut payload_val = msg.out_event.payload;
    let topic = msg.out_event.topic.clone();
    let event = msg.out_event.event.clone();

    // Encrypt only when the payload marks it as secure
    if Self::is_secure(&payload_val) && matches!(event, ChatEvent::Message) {
      if let Some(content_val) = payload_val.get_mut("content") {
        let plaintext = match content_val {
          Value::String(s) => s.clone(),
          ref other => serde_json::to_string(other).unwrap_or_default(),
        };
        if let Some(enc) = self.encrypt_str(&plaintext) {
          *content_val = Value::String(enc);
        }
      }
    }
    let frame = phx_push(&topic, &event, payload_val);
    ctx.text(frame);
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PhoenixSession {
  fn handle(&mut self, m: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match m {
      Ok(ws::Message::Text(txt)) => {
        if let Some((jr, mr, mut incoming)) = parse_phx(&txt) {
          // Keep original payload for reply echo, decrypt for broker only
          let reply = incoming.payload.clone();

          // Compute secure flag first (immutable borrow ends before mutable borrow below)
          let secure_flag = Self::is_secure(&incoming.payload);

          if let Some(obj) = incoming.payload.as_object_mut() {
            if secure_flag {
              if let Some(content_str) = obj.get("content").and_then(|v| v.as_str()) {
                if let Some(dec) = self.decrypt_str(content_str) {
                  if let Some(cv) = obj.get_mut("content") { *cv = Value::String(dec); }
                }
              }
            }
          }

          let any_event: AnyIncomingEvent = AnyIncomingEvent::from(incoming.clone());
          
          // Inject connection_id into Heartbeat if applicable
          let any_event = match any_event {
              AnyIncomingEvent::Heartbeat(mut h) => {
                  h.payload = h.payload.map(|mut p| {
                      p.connection_id = self.connection_id.clone();
                      p
                  });
                  AnyIncomingEvent::Heartbeat(h)
              },
              other => other,
          };

          let bridge_msg = match any_event {
              AnyIncomingEvent::GlobalOnline(go) => BridgeMessage { 
                  any_event: AnyIncomingEvent::GlobalOnline(go),
                  incoming_event: incoming.clone() 
              },
              AnyIncomingEvent::GlobalOffline(go) => BridgeMessage { 
                  any_event: AnyIncomingEvent::GlobalOffline(go),
                  incoming_event: incoming.clone() 
              },
              _ => BridgeMessage { any_event, incoming_event: incoming.clone() }
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
