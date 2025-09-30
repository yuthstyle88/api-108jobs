use crate::broker::helper::{ensure_room_membership, list_chat_messages};
use crate::broker::phoenix_manager::{FetchHistoryDirect, PhoenixManager};
use actix::{Context, Handler, ResponseFuture};
use lemmy_db_schema::source::chat_message::{ChatMessage, ChatMessageInsertForm};
use lemmy_db_schema::utils::DbPool;
use lemmy_db_views_chat::api::ChatMessagesResponse;
use lemmy_utils::error::FastJobResult;

impl Handler<FetchHistoryDirect> for PhoenixManager {
  type Result = ResponseFuture<FastJobResult<ChatMessagesResponse>>;

  fn handle(&mut self, msg: FetchHistoryDirect, _ctx: &mut Context<Self>) -> Self::Result {
    let pool = self.pool.clone();
    let room_id = msg.room_id.clone();
    let user_id = msg.local_user_id.clone();
    let page_cursor = msg.page_cursor.clone();
    let limit = msg.limit;
    let page_back = msg.page_back;
    let mut phoenix_manager = self.clone();

    Box::pin(async move {
      // Drain buffered messages
      let messages: Vec<ChatMessageInsertForm> = match phoenix_manager.drain_room_buffer(&room_id).await {
        Ok(messages) => messages,
        Err(e) => {
          tracing::error!("Failed to drain messages for room {}: {}", room_id, e);
          Vec::new() // Continue with empty list to avoid failing the query
        }
      };

      // Check membership via helper
      ensure_room_membership(pool.clone(), room_id.clone(), user_id.clone()).await?;

      // Persist drained messages, if any, then query
      if !messages.is_empty() {
        let mut db_pool = DbPool::Pool(&pool);
        if let Err(e) = ChatMessage::bulk_insert(&mut db_pool, &messages).await {
          tracing::error!("Failed to persist drained messages for room {}: {}", room_id, e);
          // Continue with query even if insert fails
        }
      }

      list_chat_messages(pool, room_id, page_cursor, limit, page_back).await
    })
  }
}
