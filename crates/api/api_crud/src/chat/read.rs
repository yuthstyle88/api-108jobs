use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_schema::source::chat_message::ChatMessage;
use lemmy_db_views_chat::api::{ChatRoomWithParticipants, LastMessage};
use lemmy_db_views_chat::ChatRoomView;
use lemmy_utils::error::FastJobResult;

/// GET /api/v4/chat/rooms/{id}
/// Returns the chat room along with its participants and last message (if any).
pub async fn get_chat_room(
  path: Path<String>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<ChatRoomWithParticipants>> {
  let room_id = ChatRoomId(path.into_inner());
  let mut pool = context.pool();

  // fetch room with participants
  let view = ChatRoomView::read(&mut pool, room_id.clone()).await?;

  // fetch last message for this room
  let last_message_opt = ChatMessage::last_by_room(&mut pool, room_id).await?;
  let last_message = last_message_opt.map(|m| LastMessage {
    content: m.content,
    timestamp: m.created_at.to_rfc3339(),
    sender_id: m.sender_id,
  });

  Ok(Json(ChatRoomWithParticipants {
    room: view.room,
    participants: view.participants,
    last_message,
  }))
}
