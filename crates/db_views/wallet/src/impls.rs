use crate::api::ListWalletTopupsQuery;
use crate::{WalletTopupView, WalletView};
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::newtypes::{LocalUserId, PaginationCursor, WalletTopupId};
use lemmy_db_schema::source::wallet::Wallet;
use lemmy_db_schema::source::wallet_topup::wallet_topups_keys as key;
use lemmy_db_schema::source::wallet_topup::WalletTopup;
use lemmy_db_schema::traits::{Crud, PaginationCursorBuilder};
use lemmy_db_schema::utils::{limit_fetch, paginate};
use lemmy_db_schema::{
  newtypes::WalletId,
  source::wallet::WalletModel,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{local_user, wallet, wallet_topups};
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

impl PaginationCursorBuilder for WalletTopupView {
  type CursorData = WalletTopup;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('T', self.wallet_topup.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    WalletTopup::read(pool, WalletTopupId(id)).await
  }
}

impl WalletTopupView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    wallet_topups::table
      .inner_join(local_user::table.on(wallet_topups::local_user_id.eq(local_user::id)))
  }

  /// Read a single record
  pub async fn read(pool: &mut DbPool<'_>, id: WalletTopupId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(wallet_topups::id.eq(id))
      .select((wallet_topups::all_columns, local_user::all_columns))
      .first(conn)
      .await
      .map_err(|_| FastJobErrorType::NotFound.into())
  }

  /// List topups for a given user with filters and pagination
  pub async fn list(
    pool: &mut DbPool<'_>,
    local_user_id: Option<LocalUserId>,
    cursor_data: Option<WalletTopup>,
    params: ListWalletTopupsQuery,
  ) -> FastJobResult<Vec<Self>> {
    use diesel::prelude::*;
    use lemmy_db_schema_file::schema::wallet_topups;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(params.limit)?;
    let mut query = Self::joins()
      .select((wallet_topups::all_columns, local_user::all_columns))
      .limit(limit)
      .into_boxed();

    // Filter by user (if provided)
    if let Some(uid) = local_user_id {
      query = query.filter(wallet_topups::local_user_id.eq(uid));
    }

    // Filter by status
    if let Some(ref s) = params.status {
      query = query.filter(wallet_topups::status.eq(s));
    }

    // Filter by amount range
    if let Some(min) = params.amount_min {
      query = query.filter(wallet_topups::amount.ge(min));
    }
    if let Some(max) = params.amount_max {
      query = query.filter(wallet_topups::amount.le(max));
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
