use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::flush_room_and_update_last_message;
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_schema::utils::DbPool;
use lemmy_db_views_chat::api::{ListUserChatRooms, ListUserChatRoomsResponse};
use lemmy_db_views_chat::ChatRoomView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use lemmy_utils::redis::RedisClient;
use tracing::{error, warn};

pub async fn list_chat_rooms(
  data: Query<ListUserChatRooms>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListUserChatRoomsResponse>> {
  let mut pool = context.pool();

  drain_all_buffered_messages(&mut pool, context.redis()).await;

  let list_chat_room_views = ChatRoomView::list_for_user_paginated(
    &mut pool,
    local_user_view.local_user.id,
    data.limit,
    data.page_cursor.clone(),
    None,
  )
  .await?;

  let next_page = list_chat_room_views
    .last()
    .map(PaginationCursorBuilder::to_cursor);
  let prev_page = list_chat_room_views
    .first()
    .map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListUserChatRoomsResponse {
    rooms: list_chat_room_views,
    next_page,
    prev_page,
  }))
}

async fn drain_all_buffered_messages(pool: &mut DbPool<'_>, redis_client: &RedisClient) {
  let mut redis = redis_client.clone();
  let Ok(raw_ids) = redis.smembers("chat:active_rooms").await else {
    return;
  };

  for room_id_str in raw_ids {
    let Ok(room_id) = room_id_str.parse::<ChatRoomId>() else {
      warn!("Invalid room ID in active set: {}", room_id_str);
      continue;
    };

    if let Err(e) = flush_room_and_update_last_message(pool, &mut redis, room_id).await {
      error!("Safety net flush failed for room: {}", e);
    }
  }
}
