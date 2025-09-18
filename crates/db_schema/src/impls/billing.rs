#[cfg(feature = "full")]
use crate::{
  newtypes::BillingId,
  source::billing::{Billing, BillingInsertForm, BillingUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool}
};

#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::billing;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for Billing {
  type InsertForm = BillingInsertForm;
  type UpdateForm = BillingUpdateForm;
  type IdType = BillingId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(billing::table)
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
    diesel::update(billing::table.find(id))
    .set(form)
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

#[cfg(feature = "full")]
impl Billing {
  pub async fn get_by_comment_and_status(
    pool: &mut DbPool<'_>,
    comment_id: crate::newtypes::CommentId,
    status: lemmy_db_schema_file::enums::BillingStatus,
  ) -> FastJobResult<Option<Self>> {
    use diesel::ExpressionMethods;
    let conn = &mut get_conn(pool).await?;
    let res = billing::table
      .filter(billing::comment_id.eq(comment_id))
      .filter(billing::status.eq(status))
      .first::<Self>(conn)
      .await;

    match res {
      Ok(model) => Ok(Some(model)),
      Err(diesel::result::Error::NotFound) => Ok(None),
      Err(_e) => Err(
        lemmy_utils::error::FastJobErrorType::DatabaseError
          .into(),
      ),
    }
  }
}
