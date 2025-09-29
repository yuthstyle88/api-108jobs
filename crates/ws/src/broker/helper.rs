use crate::broker::phoenix_manager::{CONNECT_TIMEOUT_SECS, JOIN_TIMEOUT_SECS};
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId, PaginationCursor};
use lemmy_db_schema::source::chat_participant::ChatParticipant;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_schema::utils::{ActualDbPool, DbPool};
use lemmy_db_views_chat::api::ChatMessagesResponse;
use lemmy_db_views_chat::ChatMessageView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use phoenix_channels_client::{Channel, ChannelStatus, Event, Payload, Socket, Topic};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub async fn connect(socket: Arc<Socket>) -> FastJobResult<Arc<Socket>> {
  // Try to connect
  match socket
    .connect(Duration::from_secs(CONNECT_TIMEOUT_SECS))
    .await
  {
    Ok(_) => Ok(socket),
    Err(e) => {
      tracing::error!("Failed to connect to socket: {}", e);
      Err(e.into())
    }
  }
}
pub async fn send_event_to_channel(channel: Arc<Channel>, event: Event, payload: Payload) {
  if let Err(e) = channel.cast(event, payload).await {
    tracing::error!("Failed to cast message: {}", e);
  }
}
pub async fn get_or_create_channel(
  channels: Arc<RwLock<HashMap<String, Arc<Channel>>>>,
  socket: Arc<Socket>,
  name: &str,
) -> FastJobResult<Arc<Channel>> {
  // Try to get existing channel
  if let Some(channel) = channels.read().await.get(name).cloned() {
    match channel.statuses().status().await {
      Ok(status) => {
        if status == ChannelStatus::Joined {
          tracing::info!("Using existing channel: {}", name);
          return Ok(channel);
        }
        // Not joined; attempt to rejoin
        if let Ok(_) = channel.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await {
          tracing::info!("Successfully rejoined channel: {}", name);
          return Ok(channel);
        }
      }
      Err(e) => {
        tracing::info!("Channel {} status check failed: {}", name, e);
      }
    }
    channels.write().await.remove(name);
  }

  // Create new channel
  let topic = Topic::from_string(name.to_string());
  let channel = socket
    .channel(topic, None)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create channel {}: {}", name, e))?;

  // Join channel
  channel
    .join(Duration::from_secs(JOIN_TIMEOUT_SECS))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to join channel {}: {}", name, e))?;

  // Store new channel
  channels
    .write()
    .await
    .insert(name.to_string(), channel.clone());
  tracing::info!("Created new channel: {}", name);
  Ok(channel)
}

/// List chat messages using cursor pagination (module-level function; no coupling to PhoenixManager)
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
