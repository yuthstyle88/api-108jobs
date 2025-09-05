use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use lemmy_db_schema::source::chat_participant::ChatParticipant;
use lemmy_db_schema::source::chat_room::ChatRoom;
use serde::{Deserialize, Serialize};

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
pub struct ListUserChatRoomsResponse {
  pub rooms: Vec<ChatRoomWithParticipants>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChatRoomResponse {
  pub room: ChatRoom,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChatRoomRequest {
  pub id: ChatRoomId,
}
