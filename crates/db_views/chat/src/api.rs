use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId, PaginationCursor, PersonId, PostId, CommentId};
use lemmy_db_schema::source::chat_participant::ChatParticipant;
use lemmy_db_schema::source::chat_room::ChatRoom;
use serde::{Deserialize, Serialize};
use lemmy_db_schema::source::workflow::Workflow;
use crate::{ChatMessageView, ChatRoomView};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserChatRooms {
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastMessage {
  pub content: String,
  pub timestamp: String,
  pub sender_id: LocalUserId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRoomWithParticipants {
  pub room: ChatRoom,
  pub participants: Vec<ChatParticipant>,
  pub last_message: Option<LastMessage>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRoomResponse {
  pub room: ChatRoomView,
  pub last_message: Option<LastMessage>,
  pub workflow: Option<Workflow>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserChatRoomsResponse {
  pub rooms: Vec<ChatRoomWithParticipants>,
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


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatRoomRequest {
  pub partner_person_id: PersonId,
  pub room_id: Option<ChatRoomId>,
  pub post_id: Option<PostId>,
  pub current_comment_id: Option<CommentId>,
  pub room_name: Option<String>,
}
