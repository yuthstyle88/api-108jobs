use crate::api::{ChatEvent, IncomingEvent, MessageStatus};
use crate::bridge_message::{BridgeMessage, OutboundMessage};
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use crate::broker::phoenix_manager::{PhoenixManager, JOIN_TIMEOUT_SECS};
use crate::broker::presence_manager::{Heartbeat, OnlineJoin};
use crate::impls::AnyIncomingEvent;
use actix::{Context, Handler, ResponseFuture};
use actix_broker::{BrokerIssue, SystemBroker};
use chrono::Utc;
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use phoenix_channels_client::{ChannelStatus, Event, Payload};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

impl Handler<BridgeMessage> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: BridgeMessage, _ctx: &mut Context<Self>) -> Self::Result {
    // Convert wire-level IncomingEvent (payload: Value) to strongly-typed AnyIncomingEvent
    let any_event: AnyIncomingEvent = msg.any_event.clone();

    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);

    // Helper to issue outbound (to internal broker) using JSON payload
    let issue_outbound =
      |room_id: ChatRoomId, event: ChatEvent, json_payload: serde_json::Value| {
        let out_event = IncomingEvent {
          room_id: room_id.clone(),
          event,
          topic: format!("room:{}", room_id),
          payload: json_payload,
        };
        self.issue_async::<SystemBroker, _>(OutboundMessage { out_event });
      };

    match any_event {
      // ---------------------- MESSAGE ----------------------
      AnyIncomingEvent::Message(ev) => {
        if let Some(mut payload) = ev.payload.clone() {
          // mark status as Sent for outbound consumers
          payload.status = Some(MessageStatus::Sent);
          let json = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);
          issue_outbound(ev.room_id.clone(), ChatEvent::Message, json);

          // 2) persist if has id
          if payload.id.is_some() {
            let mut this = self.clone();
            let insert_data = ChatMessageInsertForm {
              msg_ref_id: payload.id.clone(),
              room_id: ev.room_id.clone(),
              sender_id: payload.sender_id,
              content: payload.content.clone(),
              status: 1,
              created_at: payload.created_at.clone(),
              updated_at: None,
            };
            actix::spawn(async move {
              if let Err(e) = this.add_messages_to_room(Some(insert_data)).await {
                tracing::error!("Failed to store message in Redis: {}", e);
              }
            });
          }
        } else {
          // still notify listeners to keep flow consistent
          issue_outbound(ev.room_id, ChatEvent::Message, serde_json::Value::Null);
        }
        Box::pin(async move {})
      }

      // ---------------------- TYPING ----------------------
      AnyIncomingEvent::Typing(ev) => {
        let json = if let Some(m) = ev.payload.clone() {
          serde_json::to_value(&m).unwrap_or(serde_json::Value::Null)
        } else {
          serde_json::json!({"typing": true})
        };
        issue_outbound(ev.room_id, ChatEvent::Typing, json);
        Box::pin(async move {})
      }

      // ---------------------- UPDATE ----------------------
      AnyIncomingEvent::Update(ev) => {
        let json = ev
          .payload
          .as_ref()
          .map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null))
          .unwrap_or(serde_json::Value::Null);
        issue_outbound(ev.room_id, ChatEvent::Update, json);
        Box::pin(async move {})
      }

      // ---------------------- READ ----------------------
      AnyIncomingEvent::Read(ev) => {
        let mut json = ev
          .payload
          .as_ref()
          .map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null))
          .unwrap_or(serde_json::Value::Null);

        let mut this = self.clone();
        let room_id_clone = ev.room_id.clone();
        let payload_clone = ev.payload.clone();
        let read_at = Utc::now();

        actix::spawn(async move {
          if let Err(e) = this.handle_read_event(room_id_clone, payload_clone, Some(read_at)) {
            tracing::error!("Failed to handle read up to event: {}", e);
          }
        });
        // Add timestamp to JSON response
        if let Some(obj) = json.as_object_mut() {
          obj.insert(
            "updatedAt".to_string(),
            serde_json::Value::String(read_at.to_rfc3339()),
          );
        }

        issue_outbound(ev.room_id, ChatEvent::ReadUpTo, json); // now safe to use original
        Box::pin(async move {})
      }
      // ---------------------- READ ----------------------
      AnyIncomingEvent::ReadUpTo(ev) => {
        let mut json = ev
          .payload
          .as_ref()
          .map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null))
          .unwrap_or(serde_json::Value::Null);

        let mut this = self.clone();
        let room_id_clone = ev.room_id.clone();
        let payload_clone = ev.payload.clone();
        let read_at = Utc::now();

        actix::spawn(async move {
          if let Err(e) = this.handle_read_event(room_id_clone, payload_clone, Some(read_at)) {
            tracing::error!("Failed to handle read up to event: {}", e);
          }
        });
        // Add timestamp to JSON response
        if let Some(obj) = json.as_object_mut() {
          obj.insert(
            "updatedAt".to_string(),
            serde_json::Value::String(read_at.to_rfc3339()),
          );
        }

        issue_outbound(ev.room_id, ChatEvent::ReadUpTo, json); // now safe to use original
        Box::pin(async move {})
      }

      // ---------------------- JOIN / HEARTBEAT / ACTIVE_ROOMS / LEAVE ----------------------
      AnyIncomingEvent::Join(ev) => {
        if let Some(payload) = ev.payload.clone() {
          let channel_name = ev.topic.clone();
          let outbound_event = ev.event.clone();
          let room_id = ev.room_id.clone();
          let sender_id = payload.sender_id;
          // keep original payload as JSON once
          let content_json = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);
          let this = self.clone();

          Box::pin(async move {
            if let Ok(arc_chan) = get_or_create_channel(channels, socket, &channel_name).await {
              // Build once
              let phoenix_event = Event::from_string(outbound_event.to_string_value());
              let payload_bytes = serde_json::to_vec(&content_json).unwrap_or_default();
              let payload: Payload = Payload::binary_from_bytes(payload_bytes);

              // Try to read current status quickly (non-blocking semantics)
              let status: Option<ChannelStatus> = timeout(Duration::from_millis(300), arc_chan.statuses().status())
                .await
                .ok()
                .and_then(|res| res.ok());

              // Mark presence (always)
              let _ = this
                .presence
                .send(OnlineJoin {
                  room_id,
                  local_user_id: sender_id,
                  started_at: Utc::now(),
                })
                .await;

              // Join only if not already joined (or unknown status)
              if !matches!(status, Some(ChannelStatus::Joined)) {
                let _ = arc_chan.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await;
              }

              // Cast the event
              send_event_to_channel(arc_chan, phoenix_event, payload).await;
            }
          })
        } else {
          Box::pin(async move {})
        }
      }
      AnyIncomingEvent::Heartbeat(ev) => {
        if let Some(payload) = ev.payload.clone() {
          let sender_id = payload.sender_id;
          let this = self.clone();
          Box::pin(async move {
            tracing::debug!("Heartbeat");
            let _ = this
              .presence
              .send(Heartbeat {
                local_user_id: sender_id,
                client_time: None,
              })
              .await;
          })
        } else {
          Box::pin(async move {})
        }
      }
      AnyIncomingEvent::ActiveRooms(_ev) => Box::pin(async move {
        tracing::debug!("ActiveRooms");
      }),
      AnyIncomingEvent::Leave(_ev) => Box::pin(async move {
        tracing::debug!("Leave");
      }),

      AnyIncomingEvent::Unknown => Box::pin(async move {}),
    }
  }
}
