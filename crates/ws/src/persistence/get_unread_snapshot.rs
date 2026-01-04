use crate::broker::manager::{GetUnreadSnapshot, PhoenixManager};
use actix::{Context, Handler, ResponseFuture};
use lemmy_db_schema::source::chat_unread::ChatUnread;
use lemmy_db_schema::utils::DbPool;
use lemmy_db_views_chat::api::UnreadSnapshotItem;
use lemmy_utils::error::FastJobResult;

impl Handler<GetUnreadSnapshot> for PhoenixManager {
  type Result = ResponseFuture<FastJobResult<Vec<UnreadSnapshotItem>>>;

  fn handle(&mut self, msg: GetUnreadSnapshot, _ctx: &mut Context<Self>) -> Self::Result {
    let pool = self.pool.clone();
    let local_user_id = msg.local_user_id.clone();

    Box::pin(async move {
      let mut db = DbPool::Pool(&pool);
      let rows = ChatUnread::unread_snapshot_for_user(&mut db, local_user_id).await?;
      let items = rows
        .into_iter()
        .map(|(room_id, unread_count, last_message_id, last_message_at)| UnreadSnapshotItem {
          room_id,
          unread_count,
          last_message_id,
          last_message_at,
        })
        .collect::<Vec<_>>();
      Ok(items)
    })
  }
}
