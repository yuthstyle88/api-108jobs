use crate::{UserBankAccountView, BankView};
use diesel::{prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{LocalUserId, BankId, UserBankAccountId},
  source::{
    bank::{Bank, BankInsertForm},
    user_bank_account::{UserBankAccount, UserBankAccountInsertForm, UserBankAccountUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{banks, user_bank_accounts};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

impl UserBankAccountView {
  pub async fn create(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_id: BankId,
    account_number: String,
    account_name: String,
    is_default: Option<bool>,
    verification_image: Option<String>,
  ) -> FastJobResult<UserBankAccount> {
    let conn = &mut get_conn(pool).await?;
    
    // Validate bank exists and is active
    let bank = banks::table
      .find(bank_id)
      .filter(banks::is_active.eq(true))
      .first::<Bank>(conn)
      .await
      .map_err(|_| FastJobErrorType::InvalidField("Bank not found or inactive".to_string()))?;

    // If setting as default, unset all other defaults for this user
    if is_default.unwrap_or(false) {
      diesel::update(user_bank_accounts::table)
        .filter(user_bank_accounts::user_id.eq(user_id))
        .set(user_bank_accounts::is_default.eq(false))
        .execute(conn)
        .await?;
    }

    // Handle verification_image storage 
    let verification_image_path = if let Some(_image_data) = verification_image {
      // TODO: Implement proper image storage (save to file system/S3)
      // For now, we'll create a placeholder path
      Some(format!("verification_images/user_{}/bank_account_{}.jpg", user_id.0, bank_id.0))
    } else {
      None
    };

    let form = UserBankAccountInsertForm {
      user_id,
      bank_id,
      account_number,
      account_name,
      is_default,
      verification_image_path,
    };

    let user_bank_account = diesel::insert_into(user_bank_accounts::table)
      .values(&form)
      .get_result::<UserBankAccount>(conn)
      .await?;

    Ok(user_bank_account)
  }

  pub async fn list_by_user(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
  ) -> FastJobResult<Vec<UserBankAccountView>> {
    let conn = &mut get_conn(pool).await?;
    
    let results = user_bank_accounts::table
      .inner_join(banks::table)
      .filter(user_bank_accounts::user_id.eq(user_id))
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
    
    // Check if account belongs to user
    let account = user_bank_accounts::table
      .find(bank_account_id)
      .filter(user_bank_accounts::user_id.eq(user_id))
      .first::<UserBankAccount>(conn)
      .await
      .map_err(|_| FastJobErrorType::InvalidField("Bank account not found".to_string()))?;

    // Unset all defaults for this user
    diesel::update(user_bank_accounts::table)
      .filter(user_bank_accounts::user_id.eq(user_id))
      .set(user_bank_accounts::is_default.eq(false))
      .execute(conn)
      .await?;

    // Set this account as default
    let updated_account = diesel::update(user_bank_accounts::table)
      .filter(user_bank_accounts::id.eq(bank_account_id))
      .set((
        user_bank_accounts::is_default.eq(true),
        user_bank_accounts::updated_at.eq(chrono::Utc::now()),
      ))
      .get_result::<UserBankAccount>(conn)
      .await?;

    Ok(updated_account)
  }

  pub async fn delete(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_account_id: UserBankAccountId,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;
    
    // Check if account belongs to user and get its default status
    let account = user_bank_accounts::table
      .find(bank_account_id)
      .filter(user_bank_accounts::user_id.eq(user_id))
      .first::<UserBankAccount>(conn)
      .await
      .map_err(|_| FastJobErrorType::InvalidField("Bank account not found".to_string()))?;

    let was_default = account.is_default.unwrap_or(false);

    // Delete the account
    diesel::delete(user_bank_accounts::table)
      .filter(user_bank_accounts::id.eq(bank_account_id))
      .filter(user_bank_accounts::user_id.eq(user_id))
      .execute(conn)
      .await?;

    // If deleted account was default, set the next available account as default
    if was_default {
      if let Ok(next_account) = user_bank_accounts::table
        .filter(user_bank_accounts::user_id.eq(user_id))
        .order(user_bank_accounts::created_at.asc())
        .first::<UserBankAccount>(conn)
        .await
      {
        diesel::update(user_bank_accounts::table)
          .filter(user_bank_accounts::id.eq(next_account.id))
          .set(user_bank_accounts::is_default.eq(true))
          .execute(conn)
          .await?;
      }
    }

    Ok(true)
  }

  pub async fn list_unverified(
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Vec<UserBankAccountView>> {
    let conn = &mut get_conn(pool).await?;
    
    let results = user_bank_accounts::table
      .inner_join(banks::table)
      .filter(user_bank_accounts::is_verified.eq(false).or(user_bank_accounts::is_verified.is_null()))
      .filter(banks::is_active.eq(true))
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

  pub async fn verify(
    pool: &mut DbPool<'_>,
    bank_account_id: UserBankAccountId,
    verified: bool,
  ) -> FastJobResult<UserBankAccount> {
    let conn = &mut get_conn(pool).await?;
    
    let updated_account = diesel::update(user_bank_accounts::table)
      .filter(user_bank_accounts::id.eq(bank_account_id))
      .set((
        user_bank_accounts::is_verified.eq(verified),
        user_bank_accounts::updated_at.eq(chrono::Utc::now()),
      ))
      .get_result::<UserBankAccount>(conn)
      .await?;

    Ok(updated_account)
  }
}

impl BankView {
  pub async fn list_by_country(
    pool: &mut DbPool<'_>,
    country: Option<String>,
  ) -> FastJobResult<Vec<Bank>> {
    let conn = &mut get_conn(pool).await?;
    
    let mut query = banks::table
      .filter(banks::is_active.eq(true))
      .into_boxed();

    if let Some(country_filter) = country {
      // Only allow Thailand and Vietnam
      if !["Thailand", "Vietnam"].contains(&country_filter.as_str()) {
        return Err(FastJobErrorType::InvalidField("Only Thailand and Vietnam banks are supported".to_string()))?;
      }
      query = query.filter(banks::country.eq(country_filter));
    } else {
      // Default to showing both Thailand and Vietnam banks
      query = query.filter(banks::country.eq_any(vec!["Thailand", "Vietnam"]));
    }

    let banks = query
      .order(banks::country.asc())
      .order(banks::name.asc())
      .load::<Bank>(conn)
      .await?;

    Ok(banks)
  }

  pub async fn read(pool: &mut DbPool<'_>, bank_id: BankId) -> Result<Bank, Error> {
    let conn = &mut get_conn(pool).await?;
    banks::table
      .find(bank_id)
      .filter(banks::is_active.eq(true))
      .first::<Bank>(conn)
      .await
  }
}