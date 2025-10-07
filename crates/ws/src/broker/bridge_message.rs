use crate::api::{ChatEvent, IncomingEnvelope, IncomingEvent, MessageModel, MessageStatus};
use crate::bridge_message::{BridgeMessage, OutboundMessage};
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use crate::broker::phoenix_manager::{PhoenixManager, JOIN_TIMEOUT_SECS};
use actix::{Context, Handler, ResponseFuture};
use actix_broker::{BrokerIssue, SystemBroker};
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use phoenix_channels_client::{ChannelStatus, Event, Payload};
use serde_json;
use std::sync::Arc;
use std::time::Duration;

impl Handler<BridgeMessage> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: BridgeMessage, _ctx: &mut Context<Self>) -> Self::Result {
    // Process only messages coming from a Phoenix client; ignore ones we ourselves rebroadcast to avoid loops
    let channel_name = msg.incoming_event.topic.to_string();
    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);

    let outbound_event_cloned = msg.incoming_event.event.clone();
    let content_cloned = msg.incoming_event.payload.clone();

    let env: IncomingEnvelope = msg.incoming_event.clone().into();

    // Single-pass handling: issue outbound + persist here, no extra hop
    let mut early_return: bool = false;
    match env {
      IncomingEnvelope::Message { room_id, payload, .. } => {
        // 1) issue outbound immediately
        let payload_out = MessageModel{
          status: Some(MessageStatus::Sent),
          ..payload.clone()
        };
        let out_event = IncomingEvent {
          room_id: room_id.clone(),
          event: ChatEvent::Message,
          topic: format!("room:{}", room_id),
          payload: Some(payload_out),
        };
        self.issue_async::<SystemBroker, _>(OutboundMessage { out_event });

        // 2) persist
        if payload.id.is_some() {
          let mut this = self.clone();
          let insert_data = ChatMessageInsertForm {
            msg_ref_id: payload.id.clone(),
            room_id: room_id.clone(),
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
        early_return = true;
      }
      IncomingEnvelope::Typing { room_id, payload, .. } => {
        // Build minimal MessageModel for typing and issue outbound
        let m = MessageModel {
          sender_id: Some(payload.sender_id),
          content: Some("chat:typing".to_string()),
          typing: Some(payload.typing),
          created_at: payload.timestamp,
          ..Default::default()
        };
        let out_event = IncomingEvent {
          room_id: room_id.clone(),
          event: ChatEvent::Typing,
          topic: format!("room:{}", room_id),
          payload: Some(m),
        };
        self.issue_async::<SystemBroker, _>(OutboundMessage { out_event });
        early_return = true;
      }
      IncomingEnvelope::Update { room_id, payload, .. } => {
        let out_event = IncomingEvent {
          room_id: room_id.clone(),
          event: ChatEvent::Update,
          topic: format!("room:{}", room_id),
          payload: Some(payload.clone()),
        };
        self.issue_async::<SystemBroker, _>(OutboundMessage { out_event });
        early_return = true;
      }
      IncomingEnvelope::PhxLeave { .. } => {
        // no-op
      }
      _ => {}
    }
    if early_return {
      // We already issued outbound (and persisted if needed). Skip channel cast path.
      return Box::pin(async move {});
    }

    // Clone mapped event for async move block
    let outbound_event_for_cast = outbound_event_cloned.clone();

    Box::pin(async move {
      let arc_chan_res = get_or_create_channel(channels, socket, &channel_name).await;

      if let Ok(arc_chan) = arc_chan_res {
        let status_res = arc_chan.statuses().status().await;
        match status_res {
          Ok(status) => {
            let phoenix_event = Event::from_string(outbound_event_for_cast.to_string_value());
            let payload_bytes = serde_json::to_vec(&content_cloned).unwrap_or_else(|e| {
              tracing::error!("Failed to serialize payload to JSON: {}", e);
              Vec::new()
            });
            let payload: Payload = Payload::binary_from_bytes(payload_bytes);

            tracing::debug!(
              "PHX cast: event={} status={:?} channel={}",
              outbound_event_for_cast.to_string_value(),
              status,
              channel_name
            );
            match status {
              ChannelStatus::Joined => {
                send_event_to_channel(arc_chan, phoenix_event, payload).await;
              }
              _ => {
                let _ = arc_chan.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await;
                send_event_to_channel(arc_chan, phoenix_event, payload).await;
              }
            }
          }
          Err(_) => {
            // no-op on status error
          }
        }
      }
    })
  }
}
