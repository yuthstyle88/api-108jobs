use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_private_instance};
use app_108jobs_db_schema::newtypes::PaginationCursor;
use app_108jobs_db_schema::traits::PaginationCursorBuilder;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_rider::{api::{ListRiders, ListRidersResponse}, RiderView};
use app_108jobs_utils::error::FastJobResult;

pub async fn list_riders(
  data: Query<ListRiders>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<ListRidersResponse>> {
  // Private instance guard
  let site_view = context.site_config().get().await?.site_view;
  check_private_instance(&local_user_view, &site_view.local_site)?;

  // Decode cursor to model if provided
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(RiderView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let riders = RiderView::list(
    &mut context.pool(),
    cursor_data,
    data.page_back,
    data.limit,
    data.online_only,
  )
  .await?;

  let next_page: Option<PaginationCursor> = riders.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page: Option<PaginationCursor> = riders.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListRidersResponse {
    riders,
    next_page,
    prev_page,
  }))
}
