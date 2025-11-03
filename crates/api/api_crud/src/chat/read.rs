use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_views_chat::api::ChatRoomResponse;
use lemmy_db_views_chat::ChatRoomView;
use lemmy_utils::error::FastJobResult;

/// GET /api/v4/chat/rooms/{id}
/// Returns the chat room along with its participants, last message (if any), and current workflow status (if any and not completed/cancelled).
pub async fn get_chat_room(
  path: Path<String>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<ChatRoomResponse>> {
  let room_id = ChatRoomId(path.into_inner());
  let mut pool = context.pool();

  let view = ChatRoomView::read(&mut pool, room_id.clone()).await?;

  Ok(Json(ChatRoomResponse { room: view }))
}
