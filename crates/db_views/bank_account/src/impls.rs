use crate::BankAccountView;
use diesel::{prelude::*, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{BankAccountId, LocalUserId},
  source::{bank::Bank, user_bank_account::BankAccount},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{banks, user_bank_accounts};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl BankAccountView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    user_bank_accounts::table.inner_join(banks::table)
  }

  pub async fn read(pool: &mut DbPool<'_>, bank_account_id: BankAccountId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(user_bank_accounts::id.eq(bank_account_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
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

  pub async fn query_with_filters(
    pool: &mut DbPool<'_>,
    user_id: Option<LocalUserId>,
    verify: Option<bool>,
    order_by: Option<String>,
  ) -> FastJobResult<Vec<BankAccountView>> {
    let conn = &mut get_conn(pool).await?;

    let mut query = Self::joins()
      .select(BankAccountView::as_select())
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

    let items = query
      .select((BankAccount::as_select(), Bank::as_select()))
      .load(conn)
      .await?;

    if items.is_empty() {
      return Err(FastJobErrorType::NotFound.into());
    }

    Ok(items)
  }
}
