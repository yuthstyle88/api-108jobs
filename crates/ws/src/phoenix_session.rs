use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web_actors::ws;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use serde_json::Value;

use crate::handler::JoinRoomQuery;
use crate::{bridge_message::BridgeMessage, broker::PhoenixManager, message::RegisterClientMsg};

// ===== helpers =====
fn parse_phx(s: &str) -> Option<(Option<String>, Option<String>, String, String, Value)> {
  let v: Value = serde_json::from_str(s).ok()?;
  let a = v.as_array()?;
  if a.len() < 5 {
    return None;
  }
  Some((
    a[0].as_str().map(|x| x.to_string()),
    a[1].as_str().map(|x| x.to_string()),
    a[2].as_str()?.to_string(),
    a[3].as_str()?.to_string(),
    a[4].clone(),
  ))
}
fn phx_reply(
  jr: &Option<String>,
  mr: &Option<String>,
  topic: &str,
  status: &str,
  resp: Value,
) -> String {
  serde_json::json!([
    jr.clone().unwrap_or_default(),
    mr.clone().unwrap_or_default(),
    topic,
    "phx_reply",
    {"status": status, "response": resp}
  ])
  .to_string()
}
fn phx_push(topic: &str, event: &str, payload: Value) -> String {
  serde_json::json!([Value::Null, Value::Null, topic, event, payload]).to_string()
}

// ===== actor =====
pub struct PhoenixSession {
  pub(crate) phoenix_manager: Addr<PhoenixManager>,
  pub(crate) params: JoinRoomQuery,
  pub(crate) client_msg: RegisterClientMsg,
}
impl PhoenixSession {
  pub fn new(
    phoenix_manager: Addr<PhoenixManager>,
    params: JoinRoomQuery,
    client_msg: RegisterClientMsg,
  ) -> Self {
    Self {
      phoenix_manager,
      params,
      client_msg,
    }
  }

  fn maybe_encrypt_outbound<'a>(
    &'a self,
    _event: &str,
    messages: &'a str,
  ) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    Cow::Borrowed(messages)
  }

  fn maybe_decrypt_incoming(&self, _content: &str) -> Option<String> {
    None
  }
}

impl Actor for PhoenixSession {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.subscribe_system_sync::<BridgeMessage>(ctx);
    // Register this client/room with the manager (similar to WsSession)
    let user_id = self.client_msg.user_id;
    let room_id = self.client_msg.room_id.clone();
    self
      .phoenix_manager
      .do_send(RegisterClientMsg { user_id, room_id });
  }
}

impl Handler<BridgeMessage> for PhoenixSession {
  type Result = ();

  fn handle(&mut self, msg: BridgeMessage, ctx: &mut Self::Context) {
    // Forward only Phoenix-sourced messages destined for our room to the Phoenix client
    // if msg.channel != self.client_msg.room_id {
    //   return;
    // }

    // Convert stored JSON string to Value for Phoenix push payload
    let payload_val: Value = serde_json::from_str(&msg.messages)
      .unwrap_or_else(|_| serde_json::json!({"message": msg.messages}));
    let topic = format!("room:{}", msg.channel);
    let payload_str = payload_val.to_string();
    let outbound_payload = self.maybe_encrypt_outbound(&msg.event, &payload_str);
    // Try to keep payload as JSON when possible
    let payload = serde_json::from_str::<Value>(outbound_payload.as_ref())
      .unwrap_or_else(|_| serde_json::json!({"message": outbound_payload.as_ref()}));
    ctx.text(phx_push(&topic, &msg.event, payload));
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PhoenixSession {
  fn handle(&mut self, m: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match m {
      Ok(ws::Message::Text(txt)) => {
        if let Some((jr, mr, topic, event, payload)) = parse_phx(&txt) {
          match event.as_str() {
            "heartbeat" => ctx.text(phx_reply(
              &jr,
              &mr,
              "phoenix",
              "ok",
              serde_json::json!({"status": "alive"}),
            )),
            "phx_join" => {
              let room_opt = self.params.resolve_room_from_query_or_topic(Some(&topic));
              // Normalize reply topic to `room:<id>` for clients
              let reply_topic = if topic.starts_with("room:") {
                topic.clone()
              } else {
                format!("room:{}", topic)
              };
              if let Some(room) = room_opt {
                self.client_msg.room_id = ChatRoomId::from_channel_name(room.as_str())
                  .unwrap_or_else(|_| ChatRoomId(room));
              }
              ctx.text(phx_reply(
                &jr,
                &mr,
                &reply_topic,
                "ok",
                serde_json::json!({"status": "joined", "room": reply_topic}),
              ));
              ctx.text(phx_push(
                &reply_topic,
                "system:welcome",
                serde_json::json!({"joined": reply_topic}),
              ));
            }
            _ => {
              // Treat any other event as an application event to forward to broker
              let messages = match payload {
                Value::String(s) => self.maybe_decrypt_incoming(&s).unwrap_or(s),
                _ => payload.to_string(),
              };

              // Derive channel from topic (e.g., "room:123")
              let channel: ChatRoomId = ChatRoomId::from_channel_name(topic.as_str())
                .unwrap_or_else(|_| ChatRoomId(topic.clone()));
              let user_id: LocalUserId = match self.client_msg.user_id {
                Some(uid) => uid,
                None => {
                  // If user_id is missing, reject with error reply and do not forward
                  ctx.text(phx_reply(
                    &jr,
                    &mr,
                    &topic,
                    "error",
                    serde_json::json!({"reason": "unauthorized"}),
                  ));
                  return;
                }
              };

              let bridge_msg = BridgeMessage {
                // Treat inbound Phoenix client messages as WebSocket-originated for broker processing
                channel,
                user_id,
                event: event.clone(),
                messages,
                security_config: false,
              };
              self.issue_async::<SystemBroker, _>(bridge_msg);
              ctx.text(phx_reply(&jr, &mr, &topic, "ok", serde_json::json!({})));
            }
          }
        }
      }
      Ok(ws::Message::Ping(b)) => ctx.pong(&b),
      Ok(ws::Message::Close(r)) => ctx.close(r),
      _ => {}
    }
  }
}
