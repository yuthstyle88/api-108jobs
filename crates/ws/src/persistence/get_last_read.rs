use crate::persistence::get_last_read;
use crate::broker::manager::{GetLastRead, PhoenixManager};
use actix::{Context, Handler, ResponseFuture};
use lemmy_db_views_chat::api::LastReadResponse;
use lemmy_utils::error::FastJobResult;

impl Handler<GetLastRead> for PhoenixManager {
  type Result = ResponseFuture<FastJobResult<LastReadResponse>>;

  fn handle(&mut self, msg: GetLastRead, _ctx: &mut Context<Self>) -> Self::Result {
    let pool = self.pool.clone();
    let room_id = msg.room_id.clone();
    let local_user_id = msg.local_user_id.clone();

    Box::pin(async move { get_last_read(pool, room_id, local_user_id).await })
  }
}
