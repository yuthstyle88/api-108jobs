use crate::api::ListTopUpRequestQuery;
use crate::{TopUpRequestView, WalletView};
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::newtypes::{LocalUserId, PaginationCursor, TopUpRequestId};
use lemmy_db_schema::source::wallet::Wallet;
use lemmy_db_schema::source::top_up_request::top_up_requests_keys as key;
use lemmy_db_schema::source::top_up_request::TopUpRequest;
use lemmy_db_schema::traits::{Crud, PaginationCursorBuilder};
use lemmy_db_schema::utils::{limit_fetch, paginate};
use lemmy_db_schema::{
  newtypes::WalletId,
  source::wallet::WalletModel,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{local_user, wallet, top_up_requests};
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

  /// List topups for a given user with filters and pagination
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

    // Filter by date parts (year, month, day)
    if let Some(y) = params.year {
      query = query.filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
        "EXTRACT(YEAR FROM created_at) = {}",
        y
      )));
    }
    if let Some(m) = params.month {
      query = query.filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
        "EXTRACT(MONTH FROM created_at) = {}",
        m
      )));
    }
    if let Some(d) = params.day {
      query = query.filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
        "EXTRACT(DAY FROM created_at) = {}",
        d
      )));
    }

    let pq = paginate(
      query,
      SortDirection::Desc,
      cursor_data,
      None,
      params.page_back,
    )
    .then_order_by(key::created_at)
    .then_order_by(key::id);

    let res = pq
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)?;

    Ok(res)
  }
}
