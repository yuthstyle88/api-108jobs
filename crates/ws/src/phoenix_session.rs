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

  fn maybe_decrypt_incoming<'a>(&'a self, messages: &'a str) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    if self.secure {
      Cow::Borrowed(messages)
    } else {
      Cow::Borrowed(messages)
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
    let payload = payload_val
      .as_ref()
      .map(|val| {
        let content = self
          .maybe_encrypt_outbound(val.content.as_deref().unwrap_or(""))
          .into_owned();
        serde_json::to_value(MessageModel { content: Some(content), ..val.clone() })
          .unwrap_or(Value::Null)
      })
      .unwrap_or_else(|| serde_json::json!({ "content": Value::Null }));
    // Try to keep payload as JSON when possible
    // tracing::info!(
    //   "Outbound Phoenix push: topic={} event={} payload={}",
    //   topic,
    //   msg.out_event.event.to_string_value(),
    //   payload
    // );
    ctx.text(phx_push(&topic, &msg.out_event.event, payload));
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PhoenixSession {
  fn handle(&mut self, m: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match m {
      Ok(ws::Message::Text(txt)) => {
        if let Some((jr, mr, incoming)) = parse_phx(&txt) {
          // Build Option<MessageModel> safely (no mutation of borrowed data)
          let payload_model = incoming
              .payload
              .as_ref()
              .map(|m| {
                let content = m.content.as_deref()
                    .map(|raw| self.maybe_decrypt_incoming(raw).into_owned());
                MessageModel { content, ..m.clone() }
              });
          tracing::debug!(
            "INBOUND payload_model{:?} event {:?}",
            payload_model,
            incoming.event.clone()
          );

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
