use crate::api::{ChatEvent, ConvertError, MessageModel, MessageStatus};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::str::FromStr;
use lemmy_utils::error::FastJobError;

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
        "statusUpdate" => ChatEvent::StatusUpdate,
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
            ChatEvent::StatusUpdate => "statusUpdate",
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
        let id = value
            .get("id")
            .and_then(|v| v.as_str()).and_then(|v| v.parse::<String>().ok());

        let sender_id: i32 = value
            .get("senderId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(0);

        let content: Option<String> = value
            .get("content")
            .and_then(|v| v.as_str()).and_then(|v| v.parse::<String>().ok())
            .map(|v| Some(v))
            .unwrap_or(
                None
            );

        let status: Option<MessageStatus> = value
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending")
            .parse::<MessageStatus>().map(|v| Some(v))
            .unwrap_or(
                None
            );

        // Parse createdAt (RFC3339) into DateTime<Utc>
        let created_at: Option<DateTime<Utc>> = value
            .get("createdAt")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(MessageModel {
            id,
            sender_id,
            content,
            status,
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