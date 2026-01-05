use crate::source::pending_sender_ack::{
  PendingSenderAck, PendingSenderAckInsertForm, PendingSenderAckUpdateForm,
};
use crate::{
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::dsl::insert_into;
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema_file::schema::pending_sender_ack;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for PendingSenderAck {
  type InsertForm = PendingSenderAckInsertForm;
  type UpdateForm = PendingSenderAckUpdateForm;
  type IdType = i64;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(pending_sender_ack::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreatePendingSenderAck)
  }
  async fn update(
    _pool: &mut DbPool<'_>,
    _id: Self::IdType,
    _form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
     unimplemented!()
  }
}

impl PendingSenderAck {}
