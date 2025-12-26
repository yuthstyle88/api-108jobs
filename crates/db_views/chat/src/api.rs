use crate::{ChatMessageView, ChatRoomView};
use chrono::{DateTime, Utc};
use lemmy_db_schema::newtypes::{
  ChatRoomId, CommentId, LocalUserId, PaginationCursor, PersonId, PostId,
};
use lemmy_db_schema::source::last_read::LastRead;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserChatRooms {
  pub limit: Option<i64>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastReadQuery {
  pub room_id: ChatRoomId,
  pub peer_id: LocalUserId,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerReadQuery {
  pub peer_id: LocalUserId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRoomResponse {
  pub room: ChatRoomView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserChatRoomsResponse {
  pub rooms: Vec<ChatRoomView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChatRoomResponse {
  pub room: ChatRoomView,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChatRoomRequest {
  pub id: ChatRoomId,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// The chat messages response, mirroring SearchResponse shape
pub struct ChatMessagesResponse {
  pub results: Vec<ChatMessageView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// The last read response,
pub struct LastReadResponse {
  pub last_read: LastRead,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// The last read response,
pub struct PeerReadResponse {
  pub peer_read: LastRead,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatRoomRequest {
  pub partner_person_id: PersonId,
  pub room_id: Option<ChatRoomId>,
  pub post_id: Option<PostId>,
  pub current_comment_id: Option<CommentId>,
  pub room_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JoinRoomQuery {
  /// Phoenix Socket(..., { params: { token } }) → ?token=...
  #[serde(default)]
  pub token: Option<String>,
  /// FE อาจไม่ส่ง room มาทาง query (จะได้จาก topic ตอน phx_join)
  #[serde(alias = "roomId", alias = "room_id", alias = "room", default)]
  pub room_id: String,
  #[serde(alias = "roomName", alias = "room_name", default)]
  pub room_name: Option<String>,
  #[serde(alias = "userId", alias = "user_id", default)]
  pub local_user_id: Option<i32>,
  /// เก็บพารามิเตอร์อื่น ๆ (เช่น vsn) ป้องกัน deserialize error
  #[serde(flatten)]
  pub extra: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
  pub room_id: ChatRoomId,
  pub cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
  pub back: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct UnreadSnapshotItem {
  pub room_id: ChatRoomId,
  pub unread_count: i32,
  pub last_message_id: Option<String>,
  pub last_message_at: Option<DateTime<Utc>>,
}
