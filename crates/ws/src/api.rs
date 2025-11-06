use actix::prelude::*;
use chrono::{DateTime, Utc};
use lemmy_db_schema::newtypes::{ChatMessageRefId, ChatRoomId, LocalUserId};
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use lemmy_db_schema_file::enums::WorkFlowStatus;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

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
    #[serde(rename = "phxJoin")]
    #[default]
    PhxJoin,
    #[serde(rename = "phxLeave")]
    PhxLeave,
    #[serde(rename = "heartbeat")]
    Heartbeat,
    #[serde(rename = "chat:message")]
    Message,
    #[serde(rename = "chat:messageAck")]
    MessageAck,
    #[serde(rename = "chat:ack")]
    AckConfirm,
    #[serde(rename = "chat:sync")]
    SyncPending,
    #[serde(rename = "chat:update")]
    Update,
    #[serde(rename = "chat:read")]
    Read,
    #[serde(rename = "readUpTo")]
    ReadUpTo,
    #[serde(rename = "chat:activeRooms")]
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
    Sending,
    Retrying,
    Delivered,
    Read,
    Failed,
}

// ================= Payload structs =================
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinPayload {
    pub topic: String,
    pub sender_id: LocalUserId,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatPayload {
    pub sender_id: LocalUserId,
}#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AckConfirmPayload {
    pub sender_id: LocalUserId,
    pub room_id: ChatRoomId,
    pub client_ids: Vec<LocalUserId>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPendingPayload {
    pub sender_id: LocalUserId,
    pub room_id: ChatRoomId,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadPayload {
    pub room_id: ChatRoomId,
    pub reader_id: LocalUserId,
    pub last_read_message_id: ChatMessageRefId,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRoomPayload {
    pub room_id: ChatRoomId,
}
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MessageModel {
    pub id: Option<String>,
    pub room_id: Option<ChatRoomId>,
    pub sender_id: Option<LocalUserId>,
    pub reader_id: Option<LocalUserId>,
    pub read_last_id: Option<ChatMessageRefId>,
    pub content: Option<String>,
    pub status: Option<MessageStatus>,
    pub secure: Option<bool>,
    pub typing: Option<bool>,
    pub update_type: Option<String>,
    pub status_target: Option<WorkFlowStatus>,
    pub prev_status: Option<WorkFlowStatus>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<ReadPayload> for MessageModel {
    fn from(rp: ReadPayload) -> Self {
        Self {
            reader_id: Some(rp.reader_id),
            read_last_id: Some(rp.last_read_message_id),
            ..Default::default()
        }
    }
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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    pub payload: serde_json::Value,
}

// ================= Strongly-typed dynamic envelope =================
// All usages of IncomingEnvelope should now use GenericIncomingEvent.

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