use crate::api::{BankAccountForm, CreateBankAccount, UpdateBankAccount};
use crate::BankAccountView;
use chrono::Utc;
use diesel::dsl::count_star;
use diesel::{prelude::*, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use lemmy_db_schema::source::user_bank_account::UserBankAccountUpdateForm;
use lemmy_db_schema::traits::Crud;
use lemmy_db_schema::{
  newtypes::{BankAccountId, LocalUserId},
  source::{
    bank::Bank,
    user_bank_account::{BankAccount, UserBankAccountInsertForm},
  },
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{banks, user_bank_accounts};
use lemmy_utils::error::{
  FastJobError, FastJobErrorExt, FastJobErrorExt2, FastJobErrorType, FastJobResult,
};
use lemmy_utils::utils::validation::validate_bank_account;

impl From<(BankAccount, Bank)> for BankAccountView {
  fn from(parts: (BankAccount, Bank)) -> Self {
    BankAccountView {
      user_bank_account: parts.0,
      bank: parts.1,
    }
  }
}

impl BankAccountView {
  pub async fn update_verify(
    pool: &mut DbPool<'_>,
    bank_account_id: BankAccountId,
    verified: bool,
  ) -> FastJobResult<BankAccount> {
    // keep path usage
    let form = UserBankAccountUpdateForm {
      is_verified: Some(verified),
      updated_at: Some(Some(Utc::now())),
      ..Default::default()
    };
    let updated = BankAccount::update(pool, bank_account_id, &form).await?;
    Ok(updated)
  }

  async fn exists_for_user_by_bank_and_number(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_id: i32,
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

  pub async fn update(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    data: UpdateBankAccount,
  ) -> FastJobResult<BankAccount> {
    // Ensure ownership first in its own connection scope
    {
      let conn = &mut get_conn(pool).await?;
      let exists = diesel::select(diesel::dsl::exists(
        user_bank_accounts::table
          .filter(user_bank_accounts::id.eq(data.bank_account_id))
          .filter(user_bank_accounts::local_user_id.eq(user_id)),
      ))
      .get_result::<bool>(conn)
      .await?;

      if !exists {
        return Err(FastJobErrorType::NotFound.into());
      }
    }

    // Load current account to decide duplicate check target values
    let current: BankAccount = {
      let conn = &mut get_conn(pool).await?;
      user_bank_accounts::table
        .filter(user_bank_accounts::id.eq(data.bank_account_id))
        .filter(user_bank_accounts::local_user_id.eq(user_id))
        .first::<BankAccount>(conn)
        .await?
    };

    let target_bank_id = data.bank_id.unwrap_or(current.bank_id);
    let target_account_number_raw = data
      .account_number
      .unwrap_or_else(|| current.account_number.clone());
    let target_account_number = target_account_number_raw.trim().to_string();

    // Check for duplicates excluding this record
    if Self::exists_for_user_by_bank_and_number(
      pool,
      user_id,
      target_bank_id.0,
      &target_account_number,
      Some(data.bank_account_id),
    )
    .await?
    {
      return Err(FastJobErrorType::BankAccountAlreadyExistsForThisBank.into());
    }

    let now_time = Utc::now();

    // Compute optional verification image path if provided
    let verification_image_path = if data.verification_image.is_some() {
      Some(format!(
        "verification_images/user_{}/bank_account_{}.jpg",
        user_id.0, data.bank_account_id.0
      ))
    } else {
      None
    };

    // If setting this as default, we need a transaction to unset others first
    if matches!(data.is_default, Some(true)) {
      let conn = &mut get_conn(pool).await?;
      let bank_account_id = data.bank_account_id;
      let account_name_opt = data.account_name.clone();
      let target_account_number_tx = target_account_number.clone();
      let updated = conn
        .run_transaction(|conn| {
          async move {
            // unset all defaults for this user
            diesel::update(
              user_bank_accounts::table.filter(user_bank_accounts::local_user_id.eq(user_id)),
            )
            .set((
              user_bank_accounts::is_default.eq(Some(false)),
              user_bank_accounts::updated_at.eq(now_time),
            ))
            .execute(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateBankAccount)?;

            // apply changes to target account
            let form = UserBankAccountUpdateForm {
              bank_id: Some(target_bank_id),
              account_number: Some(target_account_number_tx),
              account_name: account_name_opt,
              is_default: Some(true),
              is_verified: None,
              updated_at: Some(Some(now_time)),
              verification_image_path: None,
            };

            let updated = diesel::update(
              user_bank_accounts::table
                .filter(user_bank_accounts::id.eq(bank_account_id))
                .filter(user_bank_accounts::local_user_id.eq(user_id)),
            )
            .set(&form)
            .get_result::<BankAccount>(conn)
            .await
            .with_fastjob_type(FastJobErrorType::CouldntUpdateBankAccount)?;

            Ok::<_, FastJobError>(updated)
          }
          .scope_boxed()
        })
        .await?;
      return Ok(updated);
    }

    // Otherwise, regular update without default juggling
    let form = UserBankAccountUpdateForm {
      bank_id: Some(target_bank_id),
      account_number: Some(target_account_number),
      account_name: data.account_name,
      is_default: data.is_default,
      is_verified: None,
      updated_at: Some(Some(now_time)),
      verification_image_path,
    };

    let conn = &mut get_conn(pool).await?;
    let updated = diesel::update(
      user_bank_accounts::table
        .filter(user_bank_accounts::id.eq(data.bank_account_id))
        .filter(user_bank_accounts::local_user_id.eq(user_id)),
    )
    .set(&form)
    .get_result::<BankAccount>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdateBankAccount)?;

    Ok(updated)
  }

  async fn is_default_account(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_account_id: BankAccountId,
  ) -> FastJobResult<bool> {
    let conn = &mut get_conn(pool).await?;

    let is_default = user_bank_accounts::table
      .find(bank_account_id)
      .filter(user_bank_accounts::local_user_id.eq(user_id))
      .select(user_bank_accounts::is_default)
      .get_result::<Option<bool>>(conn)
      .await
      .optional()?
      .flatten()
      .unwrap_or(false);

    Ok(is_default)
  }
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &mut UserBankAccountInsertForm,
  ) -> FastJobResult<BankAccount> {
    // Check duplicate by bank + account number for this user
    let acc_num_trimmed = form.account_number.trim();
    if Self::exists_for_user_by_bank_and_number(
      pool,
      form.local_user_id,
      form.bank_id.0,
      acc_num_trimmed,
      None,
    )
    .await?
    {
      return Err(FastJobErrorType::BankAccountAlreadyExistsForThisBank.into());
    }

    let conn = &mut get_conn(pool).await?;

    // Decide default based on existing accounts, then reuse Crud::create
    let user_id = form.local_user_id;
    let existing: i64 = diesel_async::RunQueryDsl::get_result(
      user_bank_accounts::table
        .filter(user_bank_accounts::local_user_id.eq(user_id))
        .select(count_star()),
      conn,
    )
    .await
    .optional()?
    .unwrap_or(0);

    let should_be_default = existing == 0;
    form.is_default = Some(should_be_default);
    // Replace account_number with trimmed version
    form.account_number = acc_num_trimmed.to_string();

    let created = BankAccount::create(pool, &form).await?;
    Ok(created)
  }

  pub async fn list_by_user(
    pool: &mut DbPool<'_>,
    local_user_id: Option<LocalUserId>,
    verify: Option<bool>,
  ) -> FastJobResult<Vec<BankAccountView>> {
    // Call list_with_filter with Some(user_id), verify=true, order_by=None
    let views = Self::query_with_filters(pool, local_user_id, verify, None).await?;
    Ok(views)
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
            user_bank_accounts::is_default.eq(Some(false)),
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
            user_bank_accounts::is_default.eq(Some(true)),
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

  pub async fn delete(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    bank_account_id: BankAccountId,
  ) -> FastJobResult<bool> {
    if Self::is_default_account(pool, user_id, bank_account_id).await? {
      return Err(
        FastJobErrorType::InvalidField(
          "Cannot delete default bank account. Please set another account as default first."
            .to_string(),
        )
        .into(),
      );
    }

    // Delete the account via Crud implementation (ownership checked above)
    let _ = BankAccount::delete(pool, bank_account_id)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntDeleteBankAccount)?;
    Ok(true)
  }
  pub async fn query_with_filters(
    pool: &mut DbPool<'_>,
    user_id: Option<LocalUserId>,
    verify: Option<bool>,
    order_by: Option<String>,
  ) -> FastJobResult<Vec<BankAccountView>> {
    let conn = &mut get_conn(pool).await?;

    let mut query = user_bank_accounts::table
      .inner_join(banks::table)
      .into_boxed();

    if let Some(id) = user_id {
      query = query.filter(user_bank_accounts::local_user_id.eq(id));
    }

    if let Some(v) = verify {
      query = query.filter(user_bank_accounts::is_verified.eq(v));
    }

    // Order logic
    match order_by.as_deref() {
      Some("created_at_asc") => {
        query = query.order(user_bank_accounts::created_at.asc());
      }
      Some("created_at_desc") => {
        query = query.order(user_bank_accounts::created_at.desc());
      }
      Some("bank_name_asc") => {
        query = query.order(banks::name.asc());
      }
      Some("bank_name_desc") => {
        query = query.order(banks::name.desc());
      }
      Some("is_default_first") => {
        query = query
          .order(user_bank_accounts::is_default.desc())
          .order(user_bank_accounts::created_at.desc());
      }
      Some("is_default_last") => {
        query = query
          .order(user_bank_accounts::is_default.asc())
          .order(user_bank_accounts::created_at.desc());
      }
      _ => {
        // Default: is_default desc, created_at desc
        query = query
          .order(user_bank_accounts::is_default.desc())
          .order(user_bank_accounts::created_at.desc());
      }
    }

    let items: Vec<(BankAccount, Bank)> = query
      .select((BankAccount::as_select(), Bank::as_select()))
      .load(conn)
      .await?;

    if items.is_empty() {
      return Err(FastJobErrorType::NotFound.into());
    }

    Ok(
      items
        .into_iter()
        .map(Into::into)
        .collect::<Vec<BankAccountView>>(),
    )
  }
}

impl TryFrom<BankAccountForm> for CreateBankAccount {
  type Error = FastJobError;

  fn try_from(data: BankAccountForm) -> Result<Self, Self::Error> {
    // Validate account number presence and format by country
    let acc_num = data.account_number.trim();
    if acc_num.is_empty() || !validate_bank_account(&data.country_id, acc_num) {
      return Err(FastJobErrorType::InvalidField("Invalid account number".to_string()).into());
    }

    // Validate account name
    if data.account_name.trim().is_empty() {
      return Err(FastJobErrorType::InvalidField("Invalid account name".to_string()).into());
    }

    Ok(CreateBankAccount {
      bank_id: data.bank_id,
      account_number: data.account_number,
      account_name: data.account_name,
      is_default: None,
      verification_image: None,
    })
  }
}
