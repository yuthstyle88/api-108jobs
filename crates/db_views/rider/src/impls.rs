use crate::RiderView;

use app_108jobs_db_schema::newtypes::DecodedCursor;
use app_108jobs_db_schema::utils::paginate;
use app_108jobs_db_schema::{
  newtypes::{LocalUserId, PaginationCursor, RiderId},
  source::rider::{rider_keys as key, Rider},
  traits::PaginationCursorBuilder,
  utils::{get_conn, Commented, DbPool},
};
use app_108jobs_db_schema_file::schema::{person, rider};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::query_builder::AsQuery;
use diesel::{self, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;

impl PaginationCursorBuilder for RiderView {
  type CursorData = Rider;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::v2_i32(self.rider.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let decoded = cursor.decode()?;

    let id = match decoded {
      DecodedCursor::I32(id) => id,
      DecodedCursor::I64(id) => id as i32,
      DecodedCursor::Composite(parts) => parts[0].1,
    };

    Rider::read(pool, RiderId(id)).await
  }
}

impl RiderView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    rider::table.inner_join(person::table.on(rider::person_id.eq(person::id)))
  }

  /// Read a single rider
  pub async fn read(pool: &mut DbPool<'_>, rider_id: RiderId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let query = Self::joins()
      .filter(rider::id.eq(rider_id))
      .select(Self::as_select());

    Commented::new(query)
      .text("RiderView::read")
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Read a rider by the owning local user id
  pub async fn read_by_user_id(pool: &mut DbPool<'_>, user_id: LocalUserId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let query = Self::joins()
      .filter(rider::user_id.eq(user_id))
      .select(Self::as_select());

    Commented::new(query)
      .text("RiderView::read_by_user_id")
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn list(
    pool: &mut DbPool<'_>,
    cursor_data: Option<Rider>,
    page_back: Option<bool>,
    limit: Option<i64>,
    verified: Option<bool>,
  ) -> FastJobResult<Vec<RiderView>> {
    use app_108jobs_db_schema_file::schema::rider;

    let conn = &mut get_conn(pool).await?;

    let limit = limit.unwrap_or(20);

    let mut query = Self::joins()
      .select(Self::as_select())
      .limit(limit)
      .into_boxed();

    // is_verified filter
    let is_verified = verified.unwrap_or(false);

    query = query.filter(rider::is_verified.eq(is_verified));

    // Active riders only
    query = query.filter(rider::is_active.eq(true));

    let paginated = paginate(query, SortDirection::Desc, cursor_data, None, page_back)
      .then_order_by(key::joined_at)
      .then_order_by(key::id);

    let query = paginated.as_query();

    Commented::new(query)
      .text("RiderView::list")
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
