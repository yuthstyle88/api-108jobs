use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::chat_participant::ChatParticipant;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::chat_room::ChatRoom
  ,
};
use lemmy_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListUserChatRooms {
  pub user_id: LocalUserId,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ListUserChatRoomsResponse {
  pub rooms: Vec<ChatRoom>,
}

pub async fn list_chat_rooms(
  data: Query<ListUserChatRooms>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<ListUserChatRoomsResponse>> {
  let limit = data.limit.unwrap_or(50);
  let rooms = ChatParticipant::list_rooms_for_user(
    &mut context.pool(),
    data.user_id,
    limit,
  )
  .await?;

  Ok(Json(ListUserChatRoomsResponse { rooms }))
}
