use crate::RiderView;

use app_108jobs_db_schema::utils::paginate;
use app_108jobs_db_schema::{
  newtypes::{PaginationCursor, RiderId},
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
    PaginationCursor::new_single('R', self.rider.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
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

  pub async fn list(
    pool: &mut DbPool<'_>,
    cursor_data: Option<Rider>,
    page_back: Option<bool>,
    limit: Option<i64>,
    online_only: Option<bool>,
  ) -> FastJobResult<Vec<RiderView>> {
    use app_108jobs_db_schema_file::schema::rider;

    let conn = &mut get_conn(pool).await?;

    let limit = limit.unwrap_or(20);

    let mut query = Self::joins()
      .select(Self::as_select())
      .limit(limit)
      .into_boxed();

    // Online filter
    if online_only.unwrap_or(false) {
      query = query.filter(rider::is_online.eq(true));
    }

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
