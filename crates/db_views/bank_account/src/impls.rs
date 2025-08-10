use crate::{BankView, UserBankAccountView};
use diesel::{prelude::*, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use lemmy_db_schema::traits::Crud;
use lemmy_db_schema::{
  newtypes::{BankId, LocalUserId, UserBankAccountId},
  source::{
    bank::Bank,
    user_bank_account::{UserBankAccount, UserBankAccountInsertForm},
  },
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{banks, user_bank_accounts};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorExt2, FastJobErrorType, FastJobResult};

impl UserBankAccountView {
  pub async fn verify(
    pool: &mut DbPool<'_>,
    bank_account_id: UserBankAccountId,
    verified: bool,
  ) -> FastJobResult<UserBankAccount> {
    // keep path usage
    use chrono::Utc;
    use lemmy_db_schema::source::user_bank_account::UserBankAccountUpdateForm;
    let form = UserBankAccountUpdateForm {
      is_verified: Some(verified),
      updated_at: Some(Utc::now()),
      ..Default::default()
    };
    let updated = UserBankAccount::update(pool, bank_account_id, &form).await?;
    Ok(updated)
  }
  async fn is_default_account(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_account_id: UserBankAccountId,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;
    use diesel::prelude::*;
    let is_default_opt: Option<bool> = diesel_async::RunQueryDsl::first(user_bank_accounts::table
      .find(bank_account_id)
      .filter(user_bank_accounts::local_user_id.eq(user_id))
      .select(user_bank_accounts::is_default), conn)
      .await
      .optional()?
      .flatten();
    Ok(is_default_opt.unwrap_or(false))
  }

  pub async fn create(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_id: BankId,
    account_number: String,
    account_name: String,
    verification_image: Option<String>,
  ) -> FastJobResult<UserBankAccount> {
    let conn = &mut get_conn(pool).await?;

    // Decide default based on existing accounts, then reuse Crud::create
    use diesel::dsl::count_star;
    use diesel::prelude::*;

    let existing: i64 = diesel_async::RunQueryDsl::get_result(user_bank_accounts::table
      .filter(user_bank_accounts::local_user_id.eq(user_id))
      .select(count_star()), conn)
      .await
      .unwrap_or(0);

    let should_be_default = existing == 0;

    let form = UserBankAccountInsertForm {
      local_user_id: user_id,
      bank_id,
      account_number,
      account_name,
      is_default: Some(should_be_default),
      verification_image_path: verification_image.map(|_| format!(
        "verification_images/user_{}/bank_account_{}.jpg",
        user_id.0, bank_id.0
      )),
    };

    let created = UserBankAccount::create(pool, &form).await?;
    Ok(created)
  }

  pub async fn list_by_user(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
  ) -> FastJobResult<Vec<UserBankAccountView>> {
    let conn = &mut get_conn(pool).await?;

    let results = user_bank_accounts::table
      .inner_join(banks::table)
      .filter(user_bank_accounts::local_user_id.eq(user_id))
      .filter(banks::is_active.eq(true))
      .order(user_bank_accounts::is_default.desc())
      .order(user_bank_accounts::created_at.desc())
      .select((UserBankAccount::as_select(), Bank::as_select()))
      .load::<(UserBankAccount, Bank)>(conn)
      .await?;

    Ok(
      results
        .into_iter()
        .map(|(user_bank_account, bank)| UserBankAccountView {
          user_bank_account,
          bank,
        })
        .collect(),
    )
  }

  pub async fn set_default(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_account_id: UserBankAccountId,
  ) -> FastJobResult<UserBankAccount> {
    let conn = &mut get_conn(pool).await?;

    // 2-step flow inside one transaction
    let updated = conn
    .transaction::<UserBankAccount, diesel::result::Error, _>(|conn| {
      Box::pin(async move {
        // 1) unset all defaults for this user
        diesel::update(
          user_bank_accounts::table
          .filter(user_bank_accounts::local_user_id.eq(user_id)),
        )
        .set(user_bank_accounts::is_default.eq(false))
        .execute(conn)
        .await?;

        // 2) set requested account to default (verify ownership)
        let rows = diesel::update(
          user_bank_accounts::table
          .filter(user_bank_accounts::id.eq(bank_account_id))
          .filter(user_bank_accounts::local_user_id.eq(user_id)),
        )
        .set(user_bank_accounts::is_default.eq(true))
        .execute(conn)
        .await?;

        if rows == 0 {
          return Err(diesel::result::Error::NotFound);
        }

        // return the newly set default
        let acc = user_bank_accounts::table
        .filter(user_bank_accounts::id.eq(bank_account_id))
        .first::<UserBankAccount>(conn)
        .await?;

        Ok(acc)
      })
    })
    .await
    .with_fastjob_type(FastJobErrorType::NotFound)?;
    Ok(updated)
  }

  pub async fn delete(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_account_id: UserBankAccountId,
  ) -> FastJobResult<bool> {

    if Self::is_default_account(pool, user_id, bank_account_id).await? {
      return Err(FastJobErrorType::InvalidField(
        "Cannot delete default bank account. Please set another account as default first.".to_string()
      ).into());
    }

    // Delete the account via Crud implementation (ownership checked above)
    let _ = UserBankAccount::delete(pool, bank_account_id).await
            .with_fastjob_type(FastJobErrorType::CouldntDeleteBankAccount)?;
    Ok(true)
  }
  pub async fn list_unverified(
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Vec<UserBankAccountView>> {
    let conn = &mut get_conn(pool).await?;

    let results = user_bank_accounts::table
      .inner_join(banks::table)
      .filter(user_bank_accounts::is_verified.eq(false))
      .order(user_bank_accounts::created_at.desc())
      .select((UserBankAccount::as_select(), Bank::as_select()))
      .load::<(UserBankAccount, Bank)>(conn)
      .await?;

    Ok(
      results
        .into_iter()
        .map(|(user_bank_account, bank)| UserBankAccountView {
          user_bank_account,
          bank,
        })
        .collect(),
    )
  }
}

impl BankView {
  pub async fn list(
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Vec<Bank>> {
    let conn = &mut get_conn(pool).await?;

    let query = banks::table
      .filter(banks::is_active.eq(true))
      .into_boxed();
    let banks = query
      .order(banks::country_id.asc())
      .order(banks::name.asc())
      .load::<Bank>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)?;
    Ok(banks)
  }
}