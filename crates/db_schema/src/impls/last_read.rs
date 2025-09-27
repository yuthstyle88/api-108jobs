use crate::newtypes::{ChatMessageId, ChatRoomId, LocalUserId};

#[cfg(feature = "full")]
use crate::{
  source::last_read::{LastRead, LastReadInsertForm, LastReadUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

#[cfg(feature = "full")]
use diesel::QueryDsl;
use diesel::{ExpressionMethods, OptionalExtension};
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::last_reads;
use lemmy_db_schema_file::schema::last_reads::dsl as lr;

#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for LastRead {
  type InsertForm = LastReadInsertForm;
  type UpdateForm = LastReadUpdateForm;
  type IdType = (LocalUserId, ChatRoomId);

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(last_reads::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      last_reads::table
        .filter(lr::user_id.eq(id.0))
        .filter(lr::room_id.eq(id.1)),
    )
    .set(form)
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

#[cfg(feature = "full")]
impl LastRead {
  pub async fn get_one(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    room_id: ChatRoomId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    last_reads::table
        .filter(lr::user_id.eq(user_id))
        .filter(lr::room_id.eq(room_id))
        .first::<Self>(conn)
        .await
        .optional()
        .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn upsert(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    room_id: ChatRoomId,
    last_read_msg_id: ChatMessageId,
  ) -> FastJobResult<Self> {
    let room_id_clone = room_id.clone();

    if let Some(_existing) = Self::get_one(pool, user_id, room_id_clone).await? {
      let form = LastReadUpdateForm {
        last_read_msg_id: Some(last_read_msg_id),
        updated_at: Some(chrono::Utc::now()),
      };
      return Self::update(pool, (user_id, room_id), &form).await;
    }

    let form = LastReadInsertForm {
      user_id,
      room_id,
      last_read_msg_id,
      updated_at: None,
    };
    Self::create(pool, &form).await
  }

}
