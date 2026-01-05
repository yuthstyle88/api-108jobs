pub mod fetch_history_direct;
pub mod get_last_read;
pub mod get_unread_snapshot;

use app_108jobs_db_schema::newtypes::{ChatRoomId, LocalUserId, PaginationCursor};
use app_108jobs_db_schema::source::chat_participant::ChatParticipant;
use app_108jobs_db_schema::source::last_read::LastRead;
use app_108jobs_db_schema::traits::PaginationCursorBuilder;
use app_108jobs_db_schema::utils::{ActualDbPool, DbPool};
use app_108jobs_db_views_chat::api::{ChatMessagesResponse, LastReadResponse};
use app_108jobs_db_views_chat::ChatMessageView;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

/// List chat messages using cursor pagination
pub async fn list_chat_messages(
  pool: ActualDbPool,
  room_id: ChatRoomId,
  page_cursor: Option<PaginationCursor>,
  limit: Option<i64>,
  page_back: Option<bool>,
) -> FastJobResult<ChatMessagesResponse> {
  let mut db_pool = DbPool::Pool(&pool);

  // Decode cursor data if provided
  let cursor_data = if let Some(ref cursor) = page_cursor {
    Some(ChatMessageView::from_cursor(cursor, &mut db_pool).await?)
  } else {
    None
  };

  // Sanitize limit: default 20, clamp to 1..=100
  let mut lim = limit.unwrap_or(20);
  if lim <= 0 {
    lim = 20;
  }
  if lim > 100 {
    lim = 100;
  }
  let lim = Some(lim);

  // If a cursor exists and direction not specified, default to paging backward (older)
  let effective_page_back = match (cursor_data.as_ref(), page_back) {
    (Some(_), None) => Some(true),
    _ => page_back,
  };

  let results =
    ChatMessageView::list_for_room(&mut db_pool, room_id, lim, cursor_data, effective_page_back)
      .await?;

  let next_page = results.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = results.first().map(PaginationCursorBuilder::to_cursor);

  Ok(ChatMessagesResponse {
    results,
    next_page,
    prev_page,
  })
}

pub async fn get_last_read(
  pool: ActualDbPool,
  room_id: ChatRoomId,
  local_user_id: LocalUserId,
) -> FastJobResult<LastReadResponse> {
  let mut db_pool = DbPool::Pool(&pool);
  let last_read = LastRead::get_one(&mut db_pool, local_user_id, room_id).await?;

  Ok(LastReadResponse { last_read })
}

pub async fn ensure_room_membership(
  pool: ActualDbPool,
  room_id: ChatRoomId,
  user_id: LocalUserId,
) -> FastJobResult<()> {
  let mut db_pool = DbPool::Pool(&pool);
  let participants =
    ChatParticipant::list_participants_for_rooms(&mut db_pool, &[room_id.clone()]).await?;
  let is_member = participants.iter().any(|p| p.member_id == user_id);
  if !is_member {
    return Err(FastJobErrorType::NotAllowed.into());
  }
  Ok(())
}
