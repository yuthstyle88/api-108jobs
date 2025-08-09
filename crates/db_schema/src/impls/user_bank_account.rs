#[cfg(feature = "full")]
use crate::{
  newtypes::UserBankAccountId,
  source::user_bank_account::{
    UserBankAccount, UserBankAccountInsertForm, UserBankAccountUpdateForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};

#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::user_bank_accounts;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for UserBankAccount {
  type InsertForm = UserBankAccountInsertForm;
  type UpdateForm = UserBankAccountUpdateForm;
  type IdType = UserBankAccountId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(user_bank_accounts::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      // No specific error type exists for bank accounts; use a generic database error wrapper.
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(user_bank_accounts::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}
