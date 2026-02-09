use crate::newtypes::TopUpRequestId;
use crate::source::top_up_request::{TopUpRequest, TopUpRequestInsertForm, TopUpRequestUpdateForm};
#[cfg(feature = "full")]
use crate::{
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::top_up_requests;
#[cfg(feature = "full")]
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for TopUpRequest {
  type InsertForm = TopUpRequestInsertForm;
  type UpdateForm = TopUpRequestUpdateForm;
  type IdType = TopUpRequestId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(top_up_requests::table)
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
    diesel::update(top_up_requests::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

#[cfg(feature = "full")]
impl TopUpRequest {
  pub async fn get_by_qr_id(pool: &mut DbPool<'_>, qr_id: &str) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    top_up_requests::table
      .filter(top_up_requests::qr_id.eq(qr_id))
      .first::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  pub async fn update_by_qr_id(
    pool: &mut DbPool<'_>,
    qr_id: String,
    form: &TopUpRequestUpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(top_up_requests::table.filter(top_up_requests::qr_id.eq(qr_id)))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}
