use crate::api::ListBankAccountQuery;
use crate::BankAccountView;
use diesel::{prelude::*, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::source::user_bank_account::user_bank_accounts_keys as key;
use lemmy_db_schema::traits::Crud;
use lemmy_db_schema::utils::{limit_fetch, paginate};
use lemmy_db_schema::{
  newtypes::{BankAccountId, LocalUserId, PaginationCursor},
  source::{bank::Bank, user_bank_account::BankAccount},
  traits::PaginationCursorBuilder,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{banks, user_bank_accounts};
use lemmy_utils::apply_date_filters;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl PaginationCursorBuilder for BankAccountView {
  type CursorData = BankAccount;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('B', self.user_bank_account.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    BankAccount::read(pool, BankAccountId(id)).await
  }
}

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

  pub async fn list(
    pool: &mut DbPool<'_>,
    local_user_id: Option<LocalUserId>,
    cursor_data: Option<BankAccount>,
    params: ListBankAccountQuery,
  ) -> FastJobResult<Vec<Self>> {
    use diesel::prelude::*;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(params.limit)?;
    let mut query = Self::joins()
      .select((BankAccount::as_select(), Bank::as_select()))
      .limit(limit)
      .into_boxed();

    // Filters
    if let Some(uid) = local_user_id {
      query = query.filter(user_bank_accounts::local_user_id.eq(uid));
    }

    if params.is_verified.is_some() {
      query = query.filter(user_bank_accounts::is_verified.eq(params.is_verified.unwrap_or(true)));
    }

    if let Some(default) = params.is_default {
      query = query.filter(user_bank_accounts::is_default.eq(default));
    }

    // Date filters via macro
    query = apply_date_filters!(query, params, "user_bank_accounts.created_at");

    // Cursor pagination via macro
    let pq = paginate(
      query,
      SortDirection::Desc,
      cursor_data,
      None,
      params.page_back,
    )
    .then_order_by(key::is_default)
    .then_order_by(key::created_at)
    .then_order_by(key::id);

    let items = pq
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)?;

    Ok(items)
  }

  pub async fn list_by_user(
    pool: &mut DbPool<'_>,
    local_user_id: Option<LocalUserId>,
    verify: Option<bool>,
  ) -> FastJobResult<Vec<BankAccountView>> {
    Self::list(
      pool,
      local_user_id,
      None,
      ListBankAccountQuery {
        limit: Some(50),
        is_verified: verify,
        is_default: None,
        year: None,
        month: None,
        day: None,
        page_cursor: None,
        page_back: None,
      },
    )
    .await
  }
}
