use crate::newtypes::{ChatMessageRefId, ChatRoomId, LocalUserId};

#[cfg(feature = "full")]
use crate::{
    source::last_read::{LastRead, LastReadInsertForm},
    utils::{get_conn, DbPool},
};

#[cfg(feature = "full")]
use diesel::QueryDsl;
use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::last_reads;
use lemmy_db_schema_file::schema::last_reads::dsl as lr;

#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
#[cfg(feature = "full")]
impl LastRead {
  pub async fn get_one(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    room_id: ChatRoomId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    last_reads::table
        .filter(lr::local_user_id.eq(local_user_id))
        .filter(lr::room_id.eq(room_id))
        .first::<Self>(conn)
        .await
        .with_fastjob_type(FastJobErrorType::LastReadNotFound)
  }

  pub async fn upsert(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    room_id: ChatRoomId,
    last_read_msg_id: ChatMessageRefId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
       diesel::insert_into(last_reads::table)
        .values(LastReadInsertForm {
          local_user_id,
          room_id,
          last_read_msg_id: last_read_msg_id.clone(),
          updated_at: None,
        })
        .on_conflict((lr::local_user_id, lr::room_id))
        .do_update()
        .set((
          lr::last_read_msg_id.eq(last_read_msg_id),
          lr::updated_at.eq(chrono::Utc::now()),
        ))
        .get_result::<Self>(conn)
        .await
        .with_fastjob_type(FastJobErrorType::CouldntSaveLastRead)
  }
}
