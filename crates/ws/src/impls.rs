use crate::api::{ChatEvent, ConvertError, IncomingEnvelope, IncomingEvent, MessageModel, MessageStatus, ReadPayload, TypingPayload};
use chrono::{DateTime, Utc};
use lemmy_db_schema::newtypes::LocalUserId;
use lemmy_db_schema_file::enums::WorkFlowStatus;
use lemmy_utils::error::FastJobError;
use serde_json::Value;
use std::str::FromStr;

impl FromStr for MessageStatus {
  type Err = ConvertError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "pending" => Ok(MessageStatus::Pending),
      "sent" => Ok(MessageStatus::Sent),
      "failed" => Ok(MessageStatus::Failed),
      other => Err(ConvertError::UnknownStatus(other.to_string())),
    }
  }
}
impl FromStr for ChatEvent {
  type Err = FastJobError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "phx_join" => ChatEvent::PhxJoin,
      "phx_leave" => ChatEvent::PhxLeave,
      "heartbeat" => ChatEvent::Heartbeat,
      "chat:message" => ChatEvent::Message,
      "chat:read" => ChatEvent::Read,
      "chat:active_rooms" => ChatEvent::ActiveRooms,
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
      ChatEvent::PhxJoin => "phx_join",
      ChatEvent::PhxLeave => "phx_leave",
      ChatEvent::Heartbeat => "heartbeat",
      ChatEvent::Message => "chat:message",
      ChatEvent::Read => "chat:read",
      ChatEvent::ActiveRooms => "chat:active_rooms",
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
impl TryFrom<Value> for MessageModel {
  type Error = FastJobError;

  fn try_from(value: Value) -> Result<Self, Self::Error> {
    // id as Option<String>
    let id: Option<String> = value
      .get("id")
      .and_then(|v| v.as_str())
      .map(|s| s.to_string());

    // senderId as Option<LocalUserId>
    let sender_id: Option<LocalUserId> = value
      .get("senderId")
      .and_then(|v| v.as_i64())
      .and_then(|n| i32::try_from(n).ok())
      .map(LocalUserId);

    let reader_id: Option<LocalUserId> = value
      .get("readerId")
      .and_then(|v| v.as_i64())
      .and_then(|n| i32::try_from(n).ok())
      .map(LocalUserId);

    let content: Option<String> = value
      .get("content")
      .and_then(|v| v.as_str())
      .map(|s| s.to_string());

    // content as Option<String>
    let read_last_id: Option<String> = value
      .get("readLastId")
      .and_then(|v| v.as_str())
      .map(|s| s.to_string());

    // status as Option<MessageStatus>; if missing or invalid -> None
    let status: Option<MessageStatus> = value
      .get("status")
      .and_then(|v| v.as_str())
      .and_then(|s| s.parse::<MessageStatus>().ok());

    // typing as Option<bool>; if missing or invalid -> None
    let typing: Option<bool> = value.get("typing").and_then(|v| v.as_bool());

    let update_type: Option<String> = value
      .get("updateType")
      .and_then(|v| v.as_str())
      .map(|s| s.to_string());

    // statusTarget as Option<WorkFlowStatus>
    let status_target: Option<WorkFlowStatus> = value
      .get("statusTarget")
      .and_then(|v| v.as_str())
      .and_then(|s| s.parse::<WorkFlowStatus>().ok());

    // prevStatus as Option<WorkFlowStatus>
    let prev_status: Option<WorkFlowStatus> = value
      .get("prevStatus")
      .and_then(|v| v.as_str())
      .and_then(|s| s.parse::<WorkFlowStatus>().ok());

    // createdAt (RFC3339) into DateTime<Utc>
    let created_at: Option<DateTime<Utc>> = value
      .get("createdAt")
      .and_then(|v| v.as_str())
      .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
      .map(|dt| dt.with_timezone(&Utc));

    Ok(MessageModel {
      id,
      sender_id,
      reader_id,
      read_last_id,
      content,
      status,
      typing,
      update_type,
      status_target,
      prev_status,
      created_at,
    })
  }
}

impl MessageModel {
  /// Serialize the message to JSON bytes (camelCase keys) for wire transport.
  /// Falls back to an empty JSON object on serialization error.
  pub fn into_bytes(&self) -> Vec<u8> {
    serde_json::to_vec(self).unwrap_or_else(|_| b"{}".to_vec())
  }
}

impl From<IncomingEvent> for IncomingEnvelope {
  fn from(ev: IncomingEvent) -> Self {
    match ev.event {
      ChatEvent::PhxJoin => IncomingEnvelope::PhxJoin {
        room_id: ev.room_id,
        topic: ev.topic,
        payload: None, // phx_join ไม่มี payload
      },
      ChatEvent::PhxLeave => IncomingEnvelope::PhxLeave {
        room_id: ev.room_id,
        topic: ev.topic,
      },
      ChatEvent::Heartbeat => IncomingEnvelope::Heartbeat {
        room_id: ev.room_id,
        topic: ev.topic,
        payload: None,
      },
      ChatEvent::Message => IncomingEnvelope::Message {
        room_id: ev.room_id,
        topic: ev.topic,
        payload: ev.payload.unwrap_or_default(), // เดิมคือ Option<MessageModel>
      },
      ChatEvent::Update => IncomingEnvelope::Update {
        room_id: ev.room_id,
        topic: ev.topic,
        payload: ev.payload.unwrap_or_default(),
      },
      ChatEvent::Read => IncomingEnvelope::Read {
        room_id: ev.room_id,
        topic: ev.topic,
        payload: ReadPayload {  sender_id: ev.payload.unwrap().sender_id.unwrap_or(LocalUserId(0))},
      },
      ChatEvent::ActiveRooms => IncomingEnvelope::ActiveRooms {
        room_id: ev.room_id,
        topic: ev.topic,
        payload: None,
      },
      ChatEvent::Typing | ChatEvent::TypingStart | ChatEvent::TypingStop => {
        IncomingEnvelope::Typing {
          room_id: ev.room_id,
          topic: ev.topic,
          payload: ev.payload
              .map(|m| TypingPayload {
                sender_id: m.sender_id.unwrap_or_default(),
                typing: m.typing.unwrap_or(false),
                timestamp: m.created_at,
              })
              .unwrap_or_default()
        }
      }
      ChatEvent::Unknown => IncomingEnvelope::PhxLeave { // หรือทำ variant Unknown แยก
        room_id: ev.room_id,
        topic: ev.topic,
      },
    }
  }
}