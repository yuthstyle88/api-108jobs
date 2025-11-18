#[cfg(feature = "full")]
use crate::{
  newtypes::BankAccountId,
  source::user_bank_account::{BankAccount, UserBankAccountInsertForm, UserBankAccountUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::dsl::count_star;
use diesel::ExpressionMethods;

use crate::newtypes::{BankId, LocalUserId};
#[cfg(feature = "full")]
use diesel::QueryDsl;
use diesel_async::scoped_futures::ScopedFutureExt;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::user_bank_accounts;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for BankAccount {
  type InsertForm = UserBankAccountInsertForm;
  type UpdateForm = UserBankAccountUpdateForm;
  type IdType = BankAccountId;

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

impl BankAccount {
  pub async fn exists_for_user_by_bank_and_number(
    pool: &mut DbPool<'_>,
    user_id: &LocalUserId,
    bank_id: &BankId,
    account_number: &str,
    exclude_id: Option<BankAccountId>,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    let mut query = user_bank_accounts::table
      .filter(user_bank_accounts::local_user_id.eq(user_id))
      .filter(user_bank_accounts::bank_id.eq(bank_id))
      .filter(user_bank_accounts::account_number.eq(account_number.to_string()))
      .into_boxed();

    if let Some(ex_id) = exclude_id {
      query = query.filter(user_bank_accounts::id.ne(ex_id));
    }

    let count: i64 = query.select(count_star()).get_result(conn).await?;

    Ok(count > 0)
  }

  pub async fn set_default(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_account_id: BankAccountId,
  ) -> FastJobResult<BankAccount> {
    let conn = &mut get_conn(pool).await?;

    // 1-step flow inside one transaction
    let updated = conn
      .run_transaction(|conn| {
        async move {
          let now_time = Utc::now();

          // 1) unset all defaults for this user
          diesel::update(
            user_bank_accounts::table.filter(user_bank_accounts::local_user_id.eq(user_id)),
          )
          .set((
            user_bank_accounts::is_default.eq(false),
            user_bank_accounts::updated_at.eq(now_time),
          ))
          .execute(conn)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateBankAccount)?;

          // 2) set requested account to default and return it
          let updated_acc = diesel::update(
            user_bank_accounts::table
              .filter(user_bank_accounts::id.eq(bank_account_id))
              .filter(user_bank_accounts::local_user_id.eq(user_id)),
          )
          .set((
            user_bank_accounts::is_default.eq(true),
            user_bank_accounts::updated_at.eq(now_time),
          ))
          .get_result::<BankAccount>(conn)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntUpdateBankAccount)?;

          Ok(updated_acc)
        }
        .scope_boxed()
      })
      .await?;
    Ok(updated)
  }

  pub async fn count_for_user(pool: &mut DbPool<'_>, user_id: &LocalUserId) -> FastJobResult<i64> {
    let conn = &mut get_conn(pool).await?;

    let count: i64 = user_bank_accounts::table
      .filter(user_bank_accounts::local_user_id.eq(user_id))
      .select(count_star())
      .get_result(conn)
      .await?;

    Ok(count)
  }
}
