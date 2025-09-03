use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::chat_participant::ChatParticipant;
use lemmy_db_schema::source::chat_room::ChatRoom;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListUserChatRooms {
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ChatRoomWithParticipants {
  pub room: ChatRoom,
  pub participants: Vec<ChatParticipant>,
}

#[derive(Debug, Serialize)]
pub struct ListUserChatRoomsResponse {
  pub rooms: Vec<ChatRoomWithParticipants>,
}

pub async fn list_chat_rooms(
  data: Query<ListUserChatRooms>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView
) -> FastJobResult<Json<ListUserChatRoomsResponse>> {
  let limit = data.limit.unwrap_or(50);
  let mut pool = context.pool();

  // 1) fetch rooms for the current user
  let rooms = ChatParticipant::list_rooms_for_user(
    &mut pool,
    local_user_view.local_user.id,
    limit,
  )
  .await?;

  // 2) fetch participants for these rooms
  let room_ids: Vec<_> = rooms.iter().map(|r| r.id.clone()).collect();
  let participants = ChatParticipant::list_participants_for_rooms(&mut pool, &room_ids).await?;

  // 3) group participants by room_id
  use std::collections::HashMap;
  let mut grouped: HashMap<String, Vec<_>> = HashMap::new();
  for p in participants {
    grouped.entry(p.room_id.to_string()).or_default().push(p);
  }

  // 4) compose response
  let rooms_with_participants = rooms
    .into_iter()
    .map(|room| {
      let parts = grouped.remove(&room.id.to_string()).unwrap_or_default();
      ChatRoomWithParticipants { room, participants: parts }
    })
    .collect();

  Ok(Json(ListUserChatRoomsResponse { rooms: rooms_with_participants }))
}
