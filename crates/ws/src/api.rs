use actix::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;


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
    #[serde(rename = "statusUpdate")]
    StatusUpdate,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    pub sender_id: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatPayload {
    pub sender_id: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadPayload {
    pub sender_id: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRoomPayload {
    pub room_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MessageModel {
    pub id: Option<String>,
    pub sender_id: i32,
    pub content: Option<String>,
    pub status: MessageStatus,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypingPayload {
    pub sender_id: i32,
    pub typing: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub sender_id: i32,
    pub room_id: String,
}

// ================= IncomingEvent =================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingEvent {
    pub event: ChatEvent,
    pub room_id: Option<ChatRoomId>,
    pub topic: String,
    pub payload: Value,
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