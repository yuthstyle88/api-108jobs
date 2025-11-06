#[cfg(feature = "full")]
use crate::{
  newtypes::WalletTopupId,
  source::wallet_topup::{WalletTopup, WalletTopupInsertForm, WalletTopupUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::wallet_topups;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for WalletTopup {
  type InsertForm = WalletTopupInsertForm;
  type UpdateForm = WalletTopupUpdateForm;
  type IdType = WalletTopupId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(wallet_topups::table)
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
    diesel::update(wallet_topups::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

#[cfg(feature = "full")]
impl WalletTopup {
  pub async fn update_by_qr_id(
    pool: &mut DbPool<'_>,
    qr_id: String,
    form: &WalletTopupUpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(wallet_topups::table.filter(wallet_topups::qr_id.eq(qr_id)))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}
