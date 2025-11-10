use crate::api::{ListTopUpRequestQuery, ListWithdrawRequestQuery};
use crate::{TopUpRequestView, WalletView, WithdrawRequestView};
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::newtypes::{
  LocalUserId, PaginationCursor, TopUpRequestId, WithdrawRequestId,
};
use lemmy_db_schema::source::top_up_request::top_up_requests_keys as t_key;
use lemmy_db_schema::source::top_up_request::TopUpRequest;
use lemmy_db_schema::source::wallet::Wallet;
use lemmy_db_schema::source::withdraw_request::withdraw_requests_keys as w_key;
use lemmy_db_schema::source::withdraw_request::WithdrawRequest;
use lemmy_db_schema::traits::{Crud, PaginationCursorBuilder};
use lemmy_db_schema::utils::{limit_fetch, paginate};
use lemmy_db_schema::{
  newtypes::WalletId,
  source::wallet::WalletModel,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{
  local_user, top_up_requests, user_bank_accounts, wallet, withdraw_requests,
};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl WalletView {
  pub async fn read(pool: &mut DbPool<'_>, wallet_id: WalletId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let wallet = wallet::table.find(wallet_id).first::<Wallet>(conn).await?;
    Ok(WalletView { wallet })
  }

  pub async fn read_by_user(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> FastJobResult<Wallet> {
    WalletModel::get_by_user(pool, local_user_id).await
  }
}

/// Reusable Cursor Pagination and Date Filter Macro
#[macro_export]
macro_rules! apply_cursor_pagination {
  ($query:ident, $cursor_data:expr, $page_back:expr, $id_col:expr) => {
    if let Some(cursor) = $cursor_data {
      if $page_back.unwrap_or(false) {
        $query = $query.filter($id_col.lt(cursor.id));
      } else {
        $query = $query.filter($id_col.gt(cursor.id));
      }
    }
  };
}

#[macro_export]
macro_rules! apply_date_filters {
  ($query:ident, $params:expr, $created_at:expr) => {{
    use diesel::dsl::sql;
    use diesel::sql_types::Bool;

    let mut q = $query;
    if let Some(y) = $params.year {
      q = q.filter(sql::<Bool>(&format!(
        "EXTRACT(YEAR FROM {}) = {}",
        $created_at, y
      )));
    }
    if let Some(m) = $params.month {
      q = q.filter(sql::<Bool>(&format!(
        "EXTRACT(MONTH FROM {}) = {}",
        $created_at, m
      )));
    }
    if let Some(d) = $params.day {
      q = q.filter(sql::<Bool>(&format!(
        "EXTRACT(DAY FROM {}) = {}",
        $created_at, d
      )));
    }
    q
  }};
}

/// Cursor-based pagination for top-ups
impl PaginationCursorBuilder for TopUpRequestView {
  type CursorData = TopUpRequest;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('T', self.top_up_request.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    TopUpRequest::read(pool, TopUpRequestId(id)).await
  }
}

impl TopUpRequestView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    top_up_requests::table
      .inner_join(local_user::table.on(top_up_requests::local_user_id.eq(local_user::id)))
  }

  /// Read a single record
  pub async fn read(pool: &mut DbPool<'_>, id: TopUpRequestId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(top_up_requests::id.eq(id))
      .select((top_up_requests::all_columns, local_user::all_columns))
      .first(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound.into())
  }

  /// List top-ups for a given user with filters and pagination
  pub async fn list(
    pool: &mut DbPool<'_>,
    local_user_id: Option<LocalUserId>,
    cursor_data: Option<TopUpRequest>,
    params: ListTopUpRequestQuery,
  ) -> FastJobResult<Vec<Self>> {
    use diesel::prelude::*;
    use lemmy_db_schema_file::schema::top_up_requests;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(params.limit)?;
    let mut query = Self::joins()
      .select((top_up_requests::all_columns, local_user::all_columns))
      .limit(limit)
      .into_boxed();

    // Filter by user (if provided)
    if let Some(uid) = local_user_id {
      query = query.filter(top_up_requests::local_user_id.eq(uid));
    }

    // Filter by status
    if let Some(ref s) = params.status {
      query = query.filter(top_up_requests::status.eq(s));
    }

    // Filter by amount range
    if let Some(min) = params.amount_min {
      query = query.filter(top_up_requests::amount.ge(min));
    }
    if let Some(max) = params.amount_max {
      query = query.filter(top_up_requests::amount.le(max));
    }

    query = apply_date_filters!(query, params, "created_at");

    let pq = paginate(
      query,
      SortDirection::Desc,
      cursor_data,
      None,
      params.page_back,
    )
    .then_order_by(t_key::created_at)
    .then_order_by(t_key::id);

    let res = pq
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)?;

    Ok(res)
  }
}

/// Cursor-based pagination for withdrawals
impl PaginationCursorBuilder for WithdrawRequestView {
  type CursorData = WithdrawRequest;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('W', self.withdraw_request.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    WithdrawRequest::read(pool, WithdrawRequestId(id)).await
  }
}

impl WithdrawRequestView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    withdraw_requests::table
      .inner_join(local_user::table.on(withdraw_requests::local_user_id.eq(local_user::id)))
      .inner_join(
        user_bank_accounts::table
          .on(withdraw_requests::user_bank_account_id.eq(user_bank_accounts::id)),
      )
  }

  /// Read a single record
  pub async fn read(pool: &mut DbPool<'_>, id: WithdrawRequestId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(withdraw_requests::id.eq(id))
      .select((
        withdraw_requests::all_columns,
        local_user::all_columns,
        user_bank_accounts::all_columns,
      ))
      .first(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound.into())
  }

  /// List withdrawals for a given user with filters and pagination
  pub async fn list(
    pool: &mut DbPool<'_>,
    local_user_id: Option<LocalUserId>,
    cursor_data: Option<WithdrawRequest>,
    params: ListWithdrawRequestQuery,
  ) -> FastJobResult<Vec<Self>> {
    use diesel::prelude::*;
    use lemmy_db_schema_file::schema::withdraw_requests;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(params.limit)?;
    let mut query = Self::joins()
      .select((
        withdraw_requests::all_columns,
        local_user::all_columns,
        user_bank_accounts::all_columns,
      ))
      .limit(limit)
      .into_boxed();

    // Filter by user (if provided)
    if let Some(uid) = local_user_id {
      query = query.filter(withdraw_requests::local_user_id.eq(uid));
    }

    // Filter by status
    if let Some(ref s) = params.status {
      query = query.filter(withdraw_requests::status.eq(s));
    }

    // Filter by amount range
    if let Some(min) = params.amount_min {
      query = query.filter(withdraw_requests::amount.ge(min));
    }
    if let Some(max) = params.amount_max {
      query = query.filter(withdraw_requests::amount.le(max));
    }

    query = apply_date_filters!(query, params, "created_at");

    // Apply cursor-based pagination using the macro
    let pq = paginate(
      query,
      SortDirection::Desc,
      cursor_data,
      None,
      params.page_back,
    )
    .then_order_by(w_key::created_at)
    .then_order_by(w_key::id);

    let res = pq
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)?;

    Ok(res)
  }
}
