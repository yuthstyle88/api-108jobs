use crate::bridge_message::{BridgeMessage, OutboundMessage};
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use crate::broker::phoenix_manager::{PhoenixManager, JOIN_TIMEOUT_SECS};
use actix::{AsyncContext, Context, Handler, Message, ResponseFuture};
use actix_broker::{BrokerIssue, SystemBroker};
use chrono::Utc;
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_schema::newtypes::LocalUserId;
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use phoenix_channels_client::{ChannelStatus, Event, Payload};
use std::sync::Arc;
use std::time::Duration;

/// List of event types that trigger an ephemeral broadcast.
/// These events are typically related to real-time chat interactions or presence updates.
const EPHEMERAL_EVENTS: &[&str] = &["chat:typing", "phx_leave"];

#[derive(Message, Clone)]
#[rtype(result = "()")]
struct DoEphemeralBroadcast {
  outbound_channel: ChatRoomId,
  event: String,
  content: Option<String>,
  store_msg: Option<ChatMessageInsertForm>,
}

impl Handler<DoEphemeralBroadcast> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: DoEphemeralBroadcast, _ctx: &mut Context<Self>) -> Self::Result {
    // Re-broadcast over broker / websocket
    self.issue_async::<SystemBroker, _>(OutboundMessage {
      channel: msg.outbound_channel,
      event: msg.event.clone(),
      messages: msg.content,
      security_config: false,
    });

    // Persist if the event is a message-type (already mapped before call if needed)
    if matches!(msg.event.as_str(), "chat:message") {
      if let Some(store_msg) = msg.store_msg {
        let msg_ref_id = store_msg.msg_ref_id.clone();
        if msg_ref_id.is_some() {
          let mut this = self.clone();
          let room_id = store_msg.room_id.clone();
          actix::spawn(async move {
            if let Err(e) = this.add_messages_to_room(room_id, store_msg).await {
              tracing::error!("Failed to store message in Redis: {}", e);
            }
          });
        }
      }
    }
  }
}

impl Handler<BridgeMessage> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: BridgeMessage, ctx: &mut Context<Self>) -> Self::Result {
    // Process only messages coming from Phoenix client; ignore ones we ourselves rebroadcast to avoid loops

    let channel_name = msg.channel.to_string();
    let event = msg.event.clone();

    let is_typing_evt = matches!(
      event.as_str(),
      "chat:typing" | "typing:start" | "typing:stop"
    );

    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);
    let message_opt = msg.messages.clone();
    let message_str = message_opt.as_deref().unwrap_or("{}").to_string();

    let chatroom_id = ChatRoomId::from_channel_name(channel_name.as_str()).unwrap_or_else(|_| {
      ChatRoomId(
        channel_name
          .strip_prefix("room:")
          .unwrap_or(&channel_name)
          .to_string(),
      )
    });

    // Parse incoming JSON payload (may be object/array/string). We expect an object for send_message.
    let incoming_val: serde_json::Value =
      serde_json::from_str(&message_str).unwrap_or_else(|_| serde_json::Value::Null);
    let obj = match incoming_val {
      serde_json::Value::Object(map) => map,
      _ => serde_json::Map::new(),
    };
    tracing::info!("READ EVT inbound: event={} payload={}", event, message_str);

    let typing_flag = if is_typing_evt {
      // Prefer explicit boolean field
      obj
        .get("typing")
        .and_then(|v| v.as_bool())
        // or inside stringified content: {"typing":true|false}
        .or_else(|| {
          obj.get("content").and_then(|v| v.as_str()).and_then(|s| {
            serde_json::from_str::<serde_json::Value>(s)
              .ok()
              .and_then(|vv| vv.get("typing").and_then(|t| t.as_bool()))
          })
        })
        .unwrap_or(false)
    } else {
      false
    };

    // Extract content only if it is meaningful (non-empty and not "{}")
    let content_text_opt: Option<&str> = obj
      .get("content")
      .and_then(|v| v.as_str())
      .map(|s| s.trim())
      .filter(|s| !s.is_empty() && *s != "{}");

    // Handle read-receipt style events early (do not treat as chat content)
    // Canonical read receipt event only
    let is_read_evt = matches!(event.as_str(), "chat:read");
    if is_read_evt {
      self.handle_read_event(&msg, chatroom_id.clone(), &obj);
      return Box::pin(async move { () });
    }

    // Extract fields with sensible fallbacks
    let msg_ref_id = obj.get("id").and_then(|v| Option::from(v.to_string()));
    let content_text = content_text_opt.unwrap_or("");
    let room_id_str: String = obj
      .get("roomId")
      .and_then(|v| v.as_str().map(|s| s.to_string()))
      .unwrap_or_else(|| chatroom_id.to_string());
    let sender_id_val = obj
      .get("senderId")
      .and_then(|v| v.as_i64())
      .map(|n| LocalUserId(n as i32));

    // Build a flat outbound payload for clients
    let mut outbound_obj = serde_json::Map::new();
    if !content_text.is_empty() {
      outbound_obj.insert(
        "content".to_string(),
        serde_json::Value::String(content_text.to_string()),
      );
    }
    if let Some(sid) = sender_id_val.filter(|v| v.0 > 0) {
    outbound_obj.insert(
      "roomId".to_string(),
      serde_json::Value::String(room_id_str.to_string()),
    );

      outbound_obj.insert(
        "senderId".to_string(),
        serde_json::Value::Number(sid.0.into()),
      );
      outbound_obj.insert(
        "status".to_string(),
        serde_json::Value::String("sent".to_string()),
      );
      if let Some(idv) = obj.get("id").cloned() {
        outbound_obj.insert("id".to_string(), idv);
      }
      if let Some(ts) = obj
          .get("createdAt")
          .cloned()
          .or_else(|| obj.get("created_at").cloned())
      {
        outbound_obj.insert("createdAt".to_string(), ts);
      } else {
        outbound_obj.insert(
          "createdAt".to_string(),
          serde_json::Value::String(Utc::now().to_rfc3339()),
        );
      }
    }
    if is_typing_evt {
      outbound_obj.insert("typing".to_string(), serde_json::Value::Bool(typing_flag));
    }
    let outbound_payload = serde_json::Value::Object(outbound_obj);
    let outbound_payload_str = outbound_payload.to_string();
    let store_msg = if let Some(sender_id) = sender_id_val {
      if !is_typing_evt && !content_text.is_empty() {
        let store_msg = ChatMessageInsertForm {
          msg_ref_id,
          room_id: chatroom_id.clone(),
          sender_id,
          content: content_text.to_string(),
          status: 1,
          created_at: Utc::now(),
          updated_at: None,
        };
        Some(store_msg)
      } else {
        None
      }
    } else {
      None
    };

    // Serialize once for casting to Phoenix channel & for broker rebroadcast
    let content = outbound_payload_str.clone();

    // Normalize channel from topic ("room:<id>") and map outbound event for clients
    let outbound_channel = ChatRoomId::from_channel_name(&channel_name).unwrap_or_else(|_| {
      ChatRoomId(
        channel_name
          .strip_prefix("room:")
          .unwrap_or(&channel_name)
          .to_string(),
      )
    });
    let outbound_event = match event.as_str() {
      "chat:message" => "chat:message",
      "chat:read"  => "chat:read",
      // pass through known page events (history flushes)
      "history_page" => "history_page",
      "chat:typing" => "chat:typing",
      "heartbeat" => "heartbeat",
      "phx_join" => "phx_join",
      // default to chat:message for other app events
      _ => "chat:message",
    }
    .to_string();

    tracing::debug!(
      "PHX bridge: inbound_event={}, outbound_event={}, channel_name={}, outbound_channel={}",
      event,
      outbound_event,
      channel_name,
      outbound_channel
    );
    tracing::debug!(
      "PHX bridge: issue_async -> WebSocket event={} channel={}",
      outbound_event,
      outbound_channel
    );

    if event.eq("chat:typing")
      || event.eq("typing:start")
      || event.eq("typing:stop")
      || event.eq("phx_leave")
      || event.eq("phx_join")
      || event.eq("heartbeat")
      || event.eq("update")
      || event.eq("room:update")
      || event.eq("chat:message")
    {
      // Run the presence check asynchronously; then hand off work back to the actor context
      let outbound_event_cloned = outbound_event.clone();
      let outbound_channel_cloned = outbound_channel.clone();
      let content_cloned = content.clone();
      let store_msg_moved = store_msg; // move into async block
      let addr = ctx.address();

      return Box::pin(async move {
        addr.do_send(DoEphemeralBroadcast {
          outbound_channel: outbound_channel_cloned,
          event: outbound_event_cloned,
          content: Some(content_cloned),
          store_msg: store_msg_moved,
        });
      });
    }
    // // Clone mapped event for async move block
    let outbound_event_for_cast = outbound_event.clone();
    Box::pin(async move {
      let arc_chan = get_or_create_channel(channels, socket, &channel_name).await;

      if let Ok(arc_chan) = arc_chan {
        let status = arc_chan.statuses().status().await;
        match status {
          Ok(status) => {
            let phoenix_event = Event::from_string(outbound_event_for_cast.clone());
            let payload: Payload = Payload::binary_from_bytes(content.into_bytes());

            tracing::debug!(
              "PHX cast: event={} status={:?} channel={}",
              outbound_event_for_cast,
              status,
              channel_name
            );

            if status == ChannelStatus::Joined {
              send_event_to_channel(arc_chan, phoenix_event, payload).await;
            } else {
              let _ = arc_chan.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await;
              send_event_to_channel(arc_chan, phoenix_event, payload).await;
            }
          }
          Err(_) => {}
        }
      }
    })
  }
}
