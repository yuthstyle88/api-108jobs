use crate::bridge_message::{BridgeMessage, GlobalOffline, GlobalOnline, OutboundMessage};
use crate::protocol::api::{ChatEvent, IncomingEvent};
use crate::protocol::impls::AnyIncomingEvent;
use crate::protocol::phx_helper::{is_base64_like, parse_phx, phx_push, phx_reply};
use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::newtypes::{ChatRoomId, LocalUserId, PostId};
use app_108jobs_utils::crypto;
use chrono::Utc;
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

// ===== actor =====
pub struct PhoenixSession {
  pub(crate) shared_key: Option<String>,
  pub(crate) local_user_id: Option<LocalUserId>,
  pub(crate) connection_id: String,
}

impl PhoenixSession {
  pub fn new(shared_key: Option<String>, local_user_id: Option<LocalUserId>) -> Self {
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
    if bytes.len() == 32 {
      Some(bytes)
    } else {
      None
    }
  }

  #[inline]
  fn encrypt_str(&self, plaintext: &str) -> Option<String> {
    let key = self.key_bytes()?;
    crypto::encrypt_string_b64(&key, plaintext).ok()
  }

  #[inline]
  fn decrypt_str(&self, b64: &str) -> Option<String> {
    if !is_base64_like(b64) {
      return None;
    }
    let key = self.key_bytes()?;
    crypto::decrypt_string_b64(&key, b64).ok()
  }

  #[inline]
  fn is_secure(payload: &Value) -> bool {
    payload
      .get("secure")
      .and_then(|v| v.as_bool())
      .unwrap_or(false)
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
      let payload = serde_json::to_value(ev.clone()).unwrap_or(Value::Null);
      let bridge_msg = BridgeMessage {
        any_event: AnyIncomingEvent::GlobalOnline(ev),
        incoming_event: IncomingEvent {
          event: ChatEvent::GlobalOnline,
          room_id: ChatRoomId("global".to_string()),
          topic: "global".to_string(),
          payload,
        },
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
          event: ChatEvent::GlobalOffline,
          room_id: ChatRoomId("global".to_string()),
          topic: "global".to_string(),
          payload: Value::Null,
        },
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
                  if let Some(cv) = obj.get_mut("content") {
                    *cv = Value::String(dec);
                  }
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
            }
            other => other,
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

pub struct DeliveryLocationSession {
  post_id: PostId,
  ctx: FastJobContext,
  last_sent: Option<String>,
}

impl DeliveryLocationSession {
  pub(crate) fn new(post_id: PostId, ctx: FastJobContext) -> Self {
    Self {
      post_id,
      ctx,
      last_sent: None,
    }
  }
}

impl Actor for DeliveryLocationSession {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    // Emit last known + start a lightweight polling loop (imitate existing utils style)
    let key = format!("delivery:current:{}", self.post_id);
    let mut redis = self.ctx.redis().clone();
    let addr = ctx.address();
    actix::spawn(async move {
      // One-time fetch of last known
      if let Ok(Some::<Value>(val)) = redis.get_value(&key).await {
        if let Ok(text) = serde_json::to_string(&val) {
          addr.do_send(EmitRaw(text));
        }
      }
    });

    // Poll every 2 seconds for changes (can be replaced with pub/sub helper later)
    let mut redis = self.ctx.redis().clone();
    let post_id = self.post_id;
    let addr = ctx.address();
    actix::spawn(async move {
      let mut tick = tokio::time::interval(Duration::from_secs(2));
      let mut last: Option<String> = None;
      loop {
        tick.tick().await;
        let key = format!("delivery:current:{}", post_id);
        if let Ok(Some::<Value>(val)) = redis.get_value(&key).await {
          if let Ok(text) = serde_json::to_string(&val) {
            if last.as_deref() != Some(&text) {
              last = Some(text.clone());
              addr.do_send(EmitMaybe { payload: text });
            }
          }
        }
      }
    });
  }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct EmitRaw(String);

impl Handler<EmitRaw> for DeliveryLocationSession {
  type Result = ();
  fn handle(&mut self, msg: EmitRaw, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(msg.0);
  }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct EmitMaybe {
  payload: String,
}

impl Handler<EmitMaybe> for DeliveryLocationSession {
  type Result = ();
  fn handle(&mut self, msg: EmitMaybe, ctx: &mut Self::Context) -> Self::Result {
    if self.last_sent.as_deref() != Some(&msg.payload) {
      self.last_sent = Some(msg.payload.clone());
      ctx.text(msg.payload);
    }
  }
}

// ============ Delivery location WS (employer/rider viewer) ============
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for DeliveryLocationSession {
  fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match item {
      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      Ok(ws::Message::Pong(_)) => {}
      Ok(ws::Message::Text(_)) => {}
      Ok(ws::Message::Binary(_)) => {}
      Ok(ws::Message::Close(reason)) => {
        ctx.close(reason);
        ctx.stop();
      }
      Ok(ws::Message::Nop) => {}
      Err(_) => {
        ctx.stop();
      }
      _ => {}
    }
  }
}
