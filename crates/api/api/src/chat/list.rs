use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_chat::api::{ListUserChatRooms, ListUserChatRoomsResponse};
use lemmy_db_views_chat::ChatRoomView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn list_chat_rooms(
  data: Query<ListUserChatRooms>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListUserChatRoomsResponse>> {
  let limit = data.limit.unwrap_or(50);
  let mut pool = context.pool();

  let list_chat_room_views =
    ChatRoomView::list_for_user(&mut pool, local_user_view.local_user.id, limit).await?;

  Ok(Json(ListUserChatRoomsResponse {
    rooms: list_chat_room_views,
  }))
}
