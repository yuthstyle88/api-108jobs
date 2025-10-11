use crate::api::{ChatEvent, IncomingEvent};
use crate::broker::phoenix_manager::{CONNECT_TIMEOUT_SECS, JOIN_TIMEOUT_SECS};
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId, PaginationCursor};
use lemmy_db_schema::source::chat_participant::ChatParticipant;
use lemmy_db_schema::source::last_read::LastRead;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_schema::utils::{ActualDbPool, DbPool};
use lemmy_db_views_chat::api::{ChatMessagesResponse, LastReadResponse};
use lemmy_db_views_chat::ChatMessageView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use phoenix_channels_client::{Channel, ChannelStatus, Event, Payload, Socket, Topic};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
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
  {
    // first fast check under read lock
    let channels_read = channels.read().await;
    if let Some(existing) = channels_read.get(name) {
      if let Ok(status) = existing.statuses().status().await {
        if status == ChannelStatus::Joined {
          tracing::debug!("Using existing channel: {}", name);
          return Ok(existing.clone());
        }
      }
    }
  }

  // acquire write lock exclusively (block others)
  let mut channels_write = channels.write().await;

  // check again inside write lock (avoid race)
  if let Some(existing) = channels_write.get(name) {
    tracing::debug!("Reusing channel (race-safe): {}", name);
    return Ok(existing.clone());
  }

  // create new channel safely
  let topic = Topic::from_string(name.to_string());
  let channel = socket
    .channel(topic, None)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create channel {}: {}", name, e))?;

  channel
    .join(Duration::from_secs(JOIN_TIMEOUT_SECS))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to join channel {}: {}", name, e))?;

  channels_write.insert(name.to_string(), channel.clone());
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

pub fn parse_phx(s: &str) -> Option<(Option<String>, Option<String>, IncomingEvent)> {
  let v: Value = serde_json::from_str(s).ok()?;
  let a = v.as_array()?;
  if a.len() < 5 {
    return None;
  }
  let jr = a.get(0).and_then(|x| x.as_str()).map(|x| x.to_string());
  let mr = a.get(1).and_then(|x| x.as_str()).map(|x| x.to_string());
  let topic = a.get(2)?.as_str()?.to_string();

  let event_str = a.get(3)?.as_str().unwrap_or("");
  let event = ChatEvent::from_str(event_str).unwrap_or(ChatEvent::Unknown);
  let payload = a.get(4)?.clone();
  let room_id: ChatRoomId =
    ChatRoomId::from_channel_name(topic.as_str()).unwrap_or_else(|_| ChatRoomId(topic.clone()));

  Some((
    jr,
    mr,
    IncomingEvent {
      event,
      topic,
      payload,
      room_id,
    },
  ))
}

pub fn phx_reply(
  jr: &Option<String>,
  mr: &Option<String>,
  topic: &str,
  status: &str,
  resp: Value,
) -> String {
  serde_json::json!([
    jr.clone().unwrap_or_default(),
    mr.clone().unwrap_or_default(),
    topic,
    "phx_reply",
    {"status": status, "response": resp}
  ])
  .to_string()
}
pub fn phx_push(topic: &str, event: &ChatEvent, payload: Value) -> String {
  serde_json::json!([Value::Null, Value::Null, topic, event, payload]).to_string()
}

/// Simple heuristic: checks whether a string *looks* like base64 ciphertext.
/// Conservative: require reasonable length and base64 characters.
pub fn is_base64_like(s: &str) -> bool {
  let trimmed = s.trim();
  if trimmed.len() < 16 {
    return false;
  } // too short to be nonce+ciphertext
  trimmed
    .chars()
    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}
