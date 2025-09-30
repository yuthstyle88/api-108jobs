use crate::broker::phoenix_manager::PhoenixManager;
use crate::message::RegisterClientMsg;
use actix::{Context, Handler};
use chrono::Utc;
use lemmy_db_schema::source::chat_participant::{ChatParticipant, ChatParticipantInsertForm};
use lemmy_db_schema::utils::DbPool;
use crate::broker::presence_manager::OnlineJoin;

impl Handler<RegisterClientMsg> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: RegisterClientMsg, _ctx: &mut Context<Self>) -> Self::Result {
    let room_id = msg.room_id.clone();
    // let room_name = msg.room_name.clone();
    let local_user_id = msg.local_user_id;

    // Immediately register this user as online in Presence
    if let Some(uid) = local_user_id {
      // LocalUserId -> i32
      self.presence.do_send(OnlineJoin { local_user_id: uid.0, started_at: Utc::now() });
    }

    // Ensure participant exists for this user in this room (create if missing)
    let participant_user_id = local_user_id;
    if let Some(uid) = participant_user_id {
      let pool_for_participant = self.pool.clone();
      let room_for_participant = room_id.clone();
      let chat_participant_form = ChatParticipantInsertForm {
        room_id: room_for_participant,
        member_id: uid,
      };
      tokio::spawn(async move {
        let mut db_pool = DbPool::Pool(&pool_for_participant);
        let _ = ChatParticipant::ensure_participant(&mut db_pool, &chat_participant_form).await;
      });
    }
  }
}
