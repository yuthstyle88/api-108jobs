use crate::api::{ChatEvent, IncomingEvent, MessageModel, MessageStatus};
use crate::bridge_message::{BridgeMessage, OutboundMessage};
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use crate::broker::phoenix_manager::{PhoenixManager, JOIN_TIMEOUT_SECS};
use actix::{Addr, AsyncContext, Context, Handler, Message, ResponseFuture};
use actix_broker::{BrokerIssue, SystemBroker};
use chrono::Utc;
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_schema::newtypes::LocalUserId;
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use phoenix_channels_client::{ChannelStatus, Event, Payload, StatusesError};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

/// List of event types that trigger an ephemeral broadcast.
/// These events are typically related to real-time chat interactions or presence updates.
const EPHEMERAL_EVENTS: &[&str] = &["chat:typing", "phx_leave"];

#[derive(Message, Clone)]
#[rtype(result = "()")]
struct DoEphemeralBroadcast {
  room_id: ChatRoomId,
  event: ChatEvent,
  out_event: MessageModel,
  store_msg: Option<ChatMessageInsertForm>,
}

impl Handler<DoEphemeralBroadcast> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: DoEphemeralBroadcast, _ctx: &mut Context<Self>) -> Self::Result {
    // Re-broadcast over broker / websocket
    let message = msg.out_event.clone();
    let message = MessageModel {
      status: MessageStatus::Sent,
      ..message
    };
    let payload = serde_json::to_value(message).unwrap();
    let out_event = IncomingEvent {
      room_id: Some(msg.room_id.clone()),
      event: msg.event.clone(),
      topic: format!("room:{}", msg.room_id),
      payload,
    };
    self.issue_async::<SystemBroker, _>(OutboundMessage { out_event });

    // Persist if the event is a message-type (already mapped before call if needed)
    match msg.event.as_str() {
      "chat:message" => {
        let mut this = self.clone();
        let room_id = msg.room_id.clone();
        actix::spawn(async move {
          if let Err(e) = this.add_messages_to_room(msg.store_msg).await {
            tracing::error!("Failed to store message in Redis: {}", e);
          }
        });
      }
      _ => {}
    }
  }
}
async fn do_send_message(addr: Addr<PhoenixManager>, msg: DoEphemeralBroadcast) -> () {
  addr.do_send(msg)
}
impl Handler<BridgeMessage> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: BridgeMessage, ctx: &mut Context<Self>) -> Self::Result {
    // Process only messages coming from Phoenix client; ignore ones we ourselves rebroadcast to avoid loops
    let channel_name = msg.incoming_event.topic.to_string();
    let event = msg.incoming_event.event.clone();
    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);

    let chatroom_id = msg.incoming_event.room_id.clone().unwrap();
    let outbound_event_cloned = msg.incoming_event.event.clone();
    let content_cloned = msg.incoming_event.payload.clone();
    let event_msg: (Option<ChatMessageInsertForm>, Option<DoEphemeralBroadcast>) = match event {
      ChatEvent::PhxJoin => (None, None),
      ChatEvent::PhxLeave => (None, None),
      ChatEvent::Heartbeat => (None, None),
      ChatEvent::Message => {
        let chat_model: Result<MessageModel, _> =
          serde_json::from_value(msg.incoming_event.payload.clone());
        match chat_model {
          Ok(m) => {
            let insert_data = ChatMessageInsertForm {
              msg_ref_id: Some(m.id.to_string()),
              room_id: chatroom_id.clone(),
              sender_id: LocalUserId(m.sender_id),
              content: Option::from(m.clone().content.unwrap().to_string()),
              status: 1,
              created_at: m.created_at.clone(),
              updated_at: None,
            };
            let broadcast = DoEphemeralBroadcast {
              room_id: chatroom_id,
              event: outbound_event_cloned.clone(),
              out_event: m.clone(),
              store_msg: Some(insert_data.clone()),
            };
            (Some(insert_data), Some(broadcast))
          }
          Err(_) => (None, None),
        }
      }
      ChatEvent::Read => (None, None),
      ChatEvent::ActiveRooms => (None, None),
      ChatEvent::Typing => (None, None),
      ChatEvent::TypingStart => (None, None),
      ChatEvent::TypingStop => (None, None),
      ChatEvent::StatusUpdate => (None, None),
      ChatEvent::Unknown => (None, None),
    };

    // Serialize once for casting to Phoenix channel & for broker rebroadcast
    let content = content_cloned.to_string();

    if let Some(event_msg) = event_msg.1 {
      // Hand off work back to the actor context
      let addr = ctx.address();
      return Box::pin(async move {
        do_send_message(addr, event_msg).await;
      });
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
            let payload: Payload = Payload::binary_from_bytes(content.into_bytes());
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
