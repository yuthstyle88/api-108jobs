use std::sync::Arc;
use std::time::Duration;
use actix::{Context, Handler, ResponseFuture};
use actix_broker::{BrokerIssue, SystemBroker};
use chrono::Utc;
use phoenix_channels_client::{ChannelStatus, Event, Payload};
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use crate::bridge_message::BridgeMessage;
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use crate::broker::phoenix_manager::{PhoenixManager, JOIN_TIMEOUT_SECS};

impl Handler<BridgeMessage> for PhoenixManager {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: BridgeMessage, _ctx: &mut Context<Self>) -> Self::Result {
        // Process only messages coming from Phoenix client; ignore ones we ourselves rebroadcast to avoid loops

        let channel_name = msg.channel.to_string();
        let user_id = msg.local_user_id.clone();
        let event = msg.event.clone();

        let socket = self.socket.clone();
        let channels = Arc::clone(&self.channels);
        let message = msg.messages.clone();

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
            serde_json::from_str(&message).unwrap_or_else(|_| serde_json::Value::Null);
        let obj = match incoming_val {
            serde_json::Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };
        tracing::info!("READ EVT inbound: event={} payload={}", event, message);
        // Handle read-receipt style events early (do not treat as chat content)
        let is_read_evt = matches!(
      event.as_str(),
      "chat:read" | "chat:read-receipt" | "read" | "message:read"
    );
        if is_read_evt {
            self.handle_read_event(&msg, chatroom_id.clone(), &obj);
            return Box::pin(async move { () });
        }

        // Extract fields with sensible fallbacks
        let msg_ref_id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| message.as_str());
        let content_text = obj
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| message.as_str());
        let room_id_str: String = obj
            .get("room_id")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .or_else(|| {
                obj
                    .get("roomId")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            })
            .unwrap_or_else(|| chatroom_id.to_string());
        let sender_id_val = obj
            .get("sender_id")
            .and_then(|v| v.as_i64())
            .or_else(|| obj.get("senderId").and_then(|v| v.as_i64()))
            .unwrap_or(user_id.0 as i64);

        // Build a flat outbound payload for clients
        let mut outbound_obj = serde_json::Map::new();
        outbound_obj.insert(
            "content".to_string(),
            serde_json::Value::String(content_text.to_string()),
        );
        outbound_obj.insert(
            "room_id".to_string(),
            serde_json::Value::String(room_id_str.to_string()),
        );
        outbound_obj.insert(
            "sender_id".to_string(),
            serde_json::Value::Number(sender_id_val.into()),
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
        let outbound_payload = serde_json::Value::Object(outbound_obj);
        let outbound_payload_str = outbound_payload.to_string();

        // Store only plain text content to DB
        let store_msg = ChatMessageInsertForm {
            msg_ref_id: msg_ref_id.to_string(),
            room_id: chatroom_id.clone(),
            sender_id: user_id,
            content: content_text.to_string(),
            status: 1,
            created_at: Utc::now(),
            updated_at: None,
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
            "send_message" | "SendMessage" => "chat:message",
            "chat:read" | "message:read" | "read" => "chat:read",
            // pass through known page events (history flushes)
            "history_page" => "history_page",
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
        tracing::debug!("PHX bridge: outbound_payload={}", content);

        tracing::debug!(
      "PHX bridge: issue_async -> WebSocket event={} channel={}",
      outbound_event,
      outbound_channel
    );
        if event.eq("typing")
            || event.eq("typing:start")
            || event.eq("typing:stop")
            || event.eq("phx_leave")
            || event.eq("update")
            || event.eq("room:update")
        {
            self.issue_async::<SystemBroker, _>(BridgeMessage {
                channel: outbound_channel,
                local_user_id: msg.local_user_id.clone(),
                event: outbound_event.clone(),
                messages: content.clone(),
                security_config: false,
            });
            return Box::pin(async move {
                tracing::debug!("PHX bridge: typing event ignored");
            });
        }
        self.issue_async::<SystemBroker, _>(BridgeMessage {
            channel: outbound_channel,
            local_user_id: msg.local_user_id.clone(),
            event: outbound_event.clone(),
            messages: content.clone(),
            security_config: false,
        });

        // Store only real chat messages (ignore read/typing/system events)
        if matches!(event.as_str(), "send_message" | "SendMessage") {
            self.add_messages_to_room(chatroom_id.clone(), store_msg);
        }
        // Clone mapped event for async move block
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