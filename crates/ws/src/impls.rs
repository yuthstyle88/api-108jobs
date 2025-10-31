use crate::api::{AckConfirmPayload, ActiveRoomPayload, ChatEvent, ConvertError, GenericIncomingEvent, HeartbeatPayload, IncomingEvent, JoinPayload, MessageModel, MessageStatus, ReadPayload, SyncPendingPayload};
use lemmy_utils::error::FastJobError;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

impl FromStr for MessageStatus {
  type Err = ConvertError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "pending" => Ok(MessageStatus::Pending),
      "sent" => Ok(MessageStatus::Sent),
      "retrying" => Ok(MessageStatus::Retrying),
      "failed" => Ok(MessageStatus::Failed),
      other => Err(ConvertError::UnknownStatus(other.to_string())),
    }
  }
}
impl FromStr for ChatEvent {
  type Err = FastJobError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      // Phoenix system events: accept both snake_case and camelCase
      "phxJoin" => ChatEvent::PhxJoin,
      "phxLeave" => ChatEvent::PhxLeave,
      "heartbeat" => ChatEvent::Heartbeat,

      // Chat app events (camelCase per FE)
      "chat:message" => ChatEvent::Message,
      "messageAck" => ChatEvent::MessageAck,
      "ackConfirm" => ChatEvent::AckConfirm,
      "sync:pending" => ChatEvent::SyncPending,
      "chat:read" => ChatEvent::Read,
      "readUpTo" => ChatEvent::ReadUpTo,
      "chat:activeRooms" => ChatEvent::ActiveRooms,
      "chat:typing" => ChatEvent::Typing,
      "typing:start" => ChatEvent::TypingStart,
      "chat:stop" => ChatEvent::TypingStop,
      "chat:update" => ChatEvent::Update,
      _ => ChatEvent::Unknown,
    })
  }
}

impl ChatEvent {
  pub fn as_str(&self) -> &'static str {
    match self {
      // Phoenix system events (snake_case)
      ChatEvent::PhxJoin => "phxJoin",
      ChatEvent::PhxLeave => "phxLeave",
      ChatEvent::Heartbeat => "heartbeat",

      // Chat app events
      ChatEvent::Message => "chat:message",
      ChatEvent::MessageAck => "messageAck",
      ChatEvent::AckConfirm => "ackConfirm",
      ChatEvent::SyncPending => "sync:pending",
      ChatEvent::Read => "chat:read",
      ChatEvent::ReadUpTo => "readUpTo",
      ChatEvent::ActiveRooms => "chat:activeRooms",
      ChatEvent::Typing => "chat:typing",
      ChatEvent::TypingStart => "typing:start",
      ChatEvent::TypingStop => "typing:stop",
      ChatEvent::Update => "chat:update",
      ChatEvent::Unknown => "unknown",
    }
  }

  pub fn to_string_value(&self) -> String {
    self.as_str().to_string()
  }
}

impl MessageModel {
  /// Serialize the message to JSON bytes (camelCase keys) for wire transport.
  /// Falls back to an empty JSON object on serialization error.
  pub fn into_bytes(&self) -> Vec<u8> {
    serde_json::to_vec(self).unwrap_or_else(|_| b"{}".to_vec())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AnyIncomingEvent {
  // --- Phoenix system events (keep first) ---
  #[serde(rename = "phxJoin")]
  Join(GenericIncomingEvent<JoinPayload>),
  #[serde(rename = "phxLeave")]
  Leave(GenericIncomingEvent<serde_json::Value>),
  #[serde(rename = "heartbeat")]
  Heartbeat(GenericIncomingEvent<HeartbeatPayload>),

  // --- Chat application events (grouped, camelCase as per FE) ---
  #[serde(rename = "chat:message")]
  Message(GenericIncomingEvent<MessageModel>),

  #[serde(rename = "messageAck")]
  MessageAck(GenericIncomingEvent<MessageModel>),

  #[serde(rename = "ackConfirm")]
  AckConfirm(GenericIncomingEvent<AckConfirmPayload>),

  #[serde(rename = "sync:pending")]
  SyncPending(GenericIncomingEvent<SyncPendingPayload>),

  #[serde(rename = "chat:read")]
  Read(GenericIncomingEvent<ReadPayload>),

  #[serde(rename = "readUpTo")]
  ReadUpTo(GenericIncomingEvent<ReadPayload>),

  #[serde(rename = "chat:activeRooms")]
  ActiveRooms(GenericIncomingEvent<ActiveRoomPayload>),

  #[serde(rename = "chat:typing")]
  Typing(GenericIncomingEvent<MessageModel>),

  #[serde(rename = "chat:update")]
  Update(GenericIncomingEvent<MessageModel>),

  // --- Fallback ---
  #[serde(other)]
  Unknown,
}

impl From<IncomingEvent> for AnyIncomingEvent {
  fn from(ev: IncomingEvent) -> Self {
    match ev.event {
      ChatEvent::PhxJoin => {
        let payload: Option<JoinPayload> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::Join(GenericIncomingEvent {
          event: ChatEvent::PhxJoin,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::PhxLeave => AnyIncomingEvent::Leave(GenericIncomingEvent {
        event: ChatEvent::PhxLeave,
        room_id: ev.room_id,
        topic: ev.topic,
        payload: None,
      }),
      ChatEvent::Heartbeat => {
        let payload: Option<HeartbeatPayload> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::Heartbeat(GenericIncomingEvent {
          event: ChatEvent::Heartbeat,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::Message => {
        let payload: Option<MessageModel> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::Message(GenericIncomingEvent {
          event: ChatEvent::Message,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::AckConfirm => {
        let payload: Option<AckConfirmPayload> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::AckConfirm(GenericIncomingEvent {
          event: ChatEvent::AckConfirm,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::SyncPending => {
        let payload: Option<SyncPendingPayload> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::SyncPending(GenericIncomingEvent {
          event: ChatEvent::SyncPending,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::Read => {
        let payload: Option<ReadPayload> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::Read(GenericIncomingEvent {
          event: ChatEvent::Read,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::ReadUpTo => {
        let payload: Option<ReadPayload> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::ReadUpTo(GenericIncomingEvent {
          event: ChatEvent::ReadUpTo,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::ActiveRooms => {
        let payload: Option<ActiveRoomPayload> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::ActiveRooms(GenericIncomingEvent {
          event: ChatEvent::ActiveRooms,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::Typing | ChatEvent::TypingStart | ChatEvent::TypingStop => {
        let payload: Option<MessageModel> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::Typing(GenericIncomingEvent {
          event: ChatEvent::Typing,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::Update => {
        let payload: Option<MessageModel> = serde_json::from_value(ev.payload.clone()).ok();
        AnyIncomingEvent::Update(GenericIncomingEvent {
          event: ChatEvent::Update,
          room_id: ev.room_id,
          topic: ev.topic,
          payload,
        })
      }
      ChatEvent::Unknown => AnyIncomingEvent::Unknown,
      _ => AnyIncomingEvent::Unknown,
    }
  }
}
