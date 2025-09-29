use actix::{Context, Handler, ResponseFuture};
use lemmy_db_schema::source::chat_message::ChatMessage;
use lemmy_db_schema::utils::DbPool;
use lemmy_db_views_chat::api::ChatMessagesResponse;
use lemmy_utils::error::FastJobResult;
use crate::broker::helper::{ensure_room_membership, list_chat_messages};
use crate::broker::phoenix_manager::{FetchHistoryDirect, PhoenixManager};

impl Handler<FetchHistoryDirect> for PhoenixManager {
    type Result = ResponseFuture<FastJobResult<ChatMessagesResponse>>;

    fn handle(&mut self, msg: FetchHistoryDirect, _ctx: &mut Context<Self>) -> Self::Result {
        // Drain buffered messages synchronously while we still have &mut self
        let to_flush = self.drain_room_buffer(&msg.room_id);

        let pool = self.pool.clone();
        let room_id = msg.room_id.clone();
        let user_id = msg.local_user_id.clone();
        let page_cursor = msg.page_cursor.clone();
        let limit = msg.limit;
        let page_back = msg.page_back;

        Box::pin(async move {
            // Check membership via helper
            ensure_room_membership(pool.clone(), room_id.clone(), user_id.clone()).await?;

            // Persist drained messages, if any, then query
            if !to_flush.is_empty() {
                let mut db_pool = DbPool::Pool(&pool);
                ChatMessage::bulk_insert(&mut db_pool, &to_flush).await?;
            }
            list_chat_messages(pool, room_id, page_cursor, limit, page_back).await
        })
    }
}