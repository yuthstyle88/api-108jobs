use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::chat_message::{ChatMessage, ChatMessageInsertForm};
use lemmy_db_views_chat::api::{ListUserChatRooms, ListUserChatRoomsResponse};
use lemmy_db_views_chat::ChatRoomView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

pub async fn list_chat_rooms(
  data: Query<ListUserChatRooms>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListUserChatRoomsResponse>> {
  let limit = data.limit.unwrap_or(50);
  let mut pool = context.pool();

  {
    let mut redis = context.redis().clone();
    let active_rooms_key = "chat:active_rooms".to_string();

    if let Ok(raw_room_ids) = redis.smembers(&active_rooms_key).await {
      for room_id_str in raw_room_ids {
        let key = format!("chat:room:{}:messages", room_id_str);
        let messages: Vec<ChatMessageInsertForm> = match redis.lrange(&key, 0, -1).await {
          Ok(messages) => messages,
          Err(e) => {
            tracing::error!("Failed to fetch messages for room {}: {}", room_id_str, e);
            continue;
          }
        };

        if messages.is_empty() {
          continue;
        }

        if let Err(e) = ChatMessage::bulk_insert(&mut pool, &messages).await {
          tracing::error!("Failed to persist drained messages for room {}: {}", room_id_str, e);
          continue;
        }

        if let Err(e) = redis.delete_key(&key).await {
          if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
            tracing::error!("Failed to delete Redis key for room {}: {}", room_id_str, e);
          }
        }
        if let Err(e) = redis.srem(&active_rooms_key, room_id_str.clone()).await {
          if !matches!(e.error_type, FastJobErrorType::RedisKeyNotFound) {
            tracing::error!("Failed to remove room {} from active rooms: {}", room_id_str, e);
          }
        }
      }
    }
  }

  let list_chat_room_views =
    ChatRoomView::list_for_user(&mut pool, local_user_view.local_user.id, limit).await?;

  Ok(Json(ListUserChatRoomsResponse {
    rooms: list_chat_room_views,
  }))
}
