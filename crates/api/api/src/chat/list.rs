use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::chat_message::ChatMessage;
use lemmy_db_schema::source::chat_participant::ChatParticipant;
use lemmy_db_views_chat::api::{
  ChatRoomWithParticipants,
  LastMessage,
  ListUserChatRooms,
  ListUserChatRoomsResponse,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

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

  // 4) compose response with last_message per room
  let mut rooms_with_participants = Vec::new();
  for room in rooms {
    let parts = grouped.remove(&room.id.to_string()).unwrap_or_default();

    let last_message_opt = ChatMessage::last_by_room(&mut pool, room.id.clone()).await.unwrap_or(None);
    let last_message = last_message_opt.map(|m| LastMessage {
      content: m.content,
      timestamp: m.created_at.to_rfc3339(),
      sender_id: m.sender_id,
    });

    rooms_with_participants.push(ChatRoomWithParticipants {
      room,
      participants: parts,
      last_message,
    });
  }

  Ok(Json(ListUserChatRoomsResponse { rooms: rooms_with_participants }))
}
