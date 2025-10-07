use actix::prelude::*;
use chrono::{DateTime, Utc};
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;
use lemmy_db_schema_file::enums::WorkFlowStatus;

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct StoreChatMessage {
    pub message: Option<ChatMessageInsertForm>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct RegisterClientMsg {
    pub local_user_id: Option<LocalUserId>,
    pub room_id: ChatRoomId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ChatEvent {
    #[serde(rename = "phx_join")]
    #[default]
    PhxJoin,
    #[serde(rename = "phx_leave")]
    PhxLeave,
    #[serde(rename = "heartbeat")]
    Heartbeat,
    #[serde(rename = "chat:message")]
    Message,
    #[serde(rename = "chat:update")]
    Update,
    #[serde(rename = "chat:read")]
    Read,
    #[serde(rename = "chat:active_rooms")]
    ActiveRooms,
    #[serde(rename = "chat:typing")]
    Typing,
    #[serde(rename = "typing:start")]
    TypingStart,
    #[serde(rename = "typing:stop")]
    TypingStop,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    #[default]
    Pending,
    Sent,
    Failed,
}

// ================= Payload structs =================
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinPayload {
    pub sender_id: LocalUserId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatPayload {
    pub sender_id: LocalUserId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadPayload {
    pub sender_id: LocalUserId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRoomPayload {
    pub room_id: ChatRoomId,
}
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MessageModel {
    pub id: Option<String>,
    pub sender_id: Option<LocalUserId>,
    pub reader_id: Option<LocalUserId>,
    pub read_last_id: Option<String>,
    pub content: Option<String>,
    pub status: Option<MessageStatus>,
    pub typing: Option<bool>,
    pub update_type: Option<String>,
    pub status_target: Option<WorkFlowStatus>,
    pub prev_status: Option<WorkFlowStatus>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TypingPayload {
    pub sender_id: LocalUserId,
    #[serde(default)]
    pub typing: bool,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub sender_id: LocalUserId,
    pub room_id: String,
}

// ================= IncomingEvent =================
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingEvent {
    pub event: ChatEvent,
    pub room_id: ChatRoomId,
    pub topic: String,
    pub payload: Option<MessageModel>,
}

// ================= Strongly-typed dynamic envelope =================
// Use a *tagged enum* so Serde picks the right payload type based on `event` automatically.
// This removes the need for `payload: Option<MessageModel>` everywhere and avoids manual downcasts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum IncomingEnvelope {
    #[serde(rename = "phx_join")]
    PhxJoin {
        room_id: ChatRoomId,
        topic: String,
        #[serde(default)]
        payload: Option<JoinPayload>,
    },
    #[serde(rename = "phx_leave")]
    PhxLeave {
        room_id: ChatRoomId,
        topic: String,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        room_id: ChatRoomId,
        topic: String,
        payload: Option<HeartbeatPayload>,
    },
    #[serde(rename = "chat:message")]
    Message {
        room_id: ChatRoomId,
        topic: String,
        payload: MessageModel,
    },
    #[serde(rename = "chat:update")]
    Update {
        room_id: ChatRoomId,
        topic: String,
        payload: MessageModel,
    },
    #[serde(rename = "chat:read")]
    Read {
        room_id: ChatRoomId,
        topic: String,
        payload: ReadPayload,
    },
    #[serde(rename = "chat:active_rooms")]
    ActiveRooms {
        room_id: ChatRoomId,
        topic: String,
        payload: Option<ActiveRoomPayload>,
    },
    #[serde(rename = "chat:typing")]
    Typing {
        room_id: ChatRoomId,
        topic: String,
        payload: TypingPayload,
    },
    #[serde(rename = "typing:start")]
    TypingStart {
        room_id: ChatRoomId,
        topic: String,
        payload: TypingPayload,
    },
    #[serde(rename = "typing:stop")]
    TypingStop {
        room_id: ChatRoomId,
        topic: String,
        payload: TypingPayload,
    },
}

impl IncomingEnvelope {
    pub fn topic(&self) -> &str {
        match self {
            IncomingEnvelope::PhxJoin { topic, .. }
            | IncomingEnvelope::PhxLeave { topic, .. }
            | IncomingEnvelope::Heartbeat { topic, .. }
            | IncomingEnvelope::Message { topic, .. }
            | IncomingEnvelope::Update { topic, .. }
            | IncomingEnvelope::Read { topic, .. }
            | IncomingEnvelope::ActiveRooms { topic, .. }
            | IncomingEnvelope::Typing { topic, .. }
            | IncomingEnvelope::TypingStart { topic, .. }
            | IncomingEnvelope::TypingStop { topic, .. } => topic.as_str(),
        }
    }
}

// Optional: a generic fall-back form if you still need a single struct with a generic payload.
// This is useful when you want to parse first, then downcast payload by matching `event` yourself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericIncomingEvent<T = serde_json::Value> {
    pub event: ChatEvent,
    pub room_id: ChatRoomId,
    pub topic: String,
    #[serde(default)]
    pub payload: Option<T>,
}

// ================= AppEvent (normalized for server) =================
#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("invalid uuid: {0}")]
    InvalidUuid(String),
    #[error("unknown status: {0}")]
    UnknownStatus(String),
    #[error("missing field or invalid payload for event {0}")]
    InvalidPayload(&'static str),
}