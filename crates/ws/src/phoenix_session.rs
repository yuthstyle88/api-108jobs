use crate::api::{IncomingEvent, MessageModel};
use crate::bridge_message::OutboundMessage;
use crate::broker::helper::{parse_phx, phx_push, phx_reply};
use crate::broker::phoenix_manager::PhoenixManager;
use crate::broker::presence_manager::{OnlineJoin, OnlineLeave, PresenceManager};
use crate::handler::JoinRoomQuery;
use crate::{api::RegisterClientMsg, bridge_message::BridgeMessage};
use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use chrono::Utc;
use serde_json::Value;

// ===== actor =====
pub struct PhoenixSession {
  pub(crate) phoenix_manager: Addr<PhoenixManager>,
  pub(crate) presence_manager: Addr<PresenceManager>,
  #[allow(dead_code)]
  pub(crate) params: JoinRoomQuery,
  pub(crate) client_msg: RegisterClientMsg,
  pub(crate) secure: bool,
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
      secure: false,
    }
  }

  fn maybe_encrypt_outbound<'a>(&'a self, messages: &'a str) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    Cow::Borrowed(messages)
  }

  fn maybe_decrypt_incoming(&self, content: &str) -> serde_json::error::Result<Value> {
    if self.secure {
      serde_json::from_str(content)
    } else {
      serde_json::from_str(content)
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
    // Convert stored JSON string to Value for Phoenix push payload
    let payload_val = msg.out_event.payload;
    let topic = msg.out_event.topic;
    let payload = match payload_val {
      None => {
        serde_json::json!({"content": Value::Null})
      }
      Some(val) => {
        // Hold the owned content so the reference lives long enough
        let content_owned: String = val.content.clone().unwrap_or_default();
        // Encrypt (may return Cow); take ownership to avoid lifetime issues
        let encrypted_owned: String = self
          .maybe_encrypt_outbound(&content_owned)
          .into_owned();

        let payload = MessageModel {
          content: Some(encrypted_owned),
          ..val.clone()
        };
        serde_json::to_value(payload).unwrap_or_else(|_| Value::Null)
      }
    };
    // Try to keep payload as JSON when possible
    tracing::info!(
      "Outbound Phoenix push: topic={} event={} payload={}",
      topic,
      msg.out_event.event.to_string_value(),
      payload
    );
    ctx.text(phx_push(&topic, &msg.out_event.event, payload));
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PhoenixSession {
  fn handle(&mut self, m: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match m {
      Ok(ws::Message::Text(txt)) => {
        if let Some((jr, mr, incoming)) = parse_phx(&txt) {
          // Build Option<MessageModel> safely (no mutation of borrowed data)
          let payload_model: Option<MessageModel> = if let Some(msg) = &incoming.payload {
            let raw = msg.content.as_deref().unwrap_or("");
            let decrypted = self.maybe_decrypt_incoming(raw);
            let decrypted_str = match decrypted {
              Ok(v) => v.to_string(),
              Err(e) => {
                tracing::error!("Error decrypting message: {:?}", e);
                "".to_string()
              }
            };
            Some(MessageModel {
              content: Some(decrypted_str),
              ..msg.clone()
            })
          } else {
            None
          };

          let parse_data = IncomingEvent {
            event: incoming.event.clone(),
            room_id: incoming.room_id.clone(),
            topic: incoming.topic.clone(),
            payload: payload_model,
          };

          let bridge_msg = BridgeMessage {
            incoming_event: parse_data,
          };
          self.issue_async::<SystemBroker, _>(bridge_msg);
          ctx.text(phx_reply(
            &jr,
            &mr,
            &incoming.topic,
            "ok",
            serde_json::json!({}),
          ));
        }
      }
      Ok(ws::Message::Ping(b)) => ctx.pong(&b),
      Ok(ws::Message::Close(r)) => ctx.close(r),
      _ => {}
    }
  }
}
