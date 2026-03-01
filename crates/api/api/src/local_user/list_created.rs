use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_fetch_limit};
use app_108jobs_db_schema::traits::PaginationCursorBuilder;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::{
  logistics::{load_logistics_for_post_views, LogisticsViewer},
  PostView,
};
use app_108jobs_db_views_site::api::{ListPersonCreated, ListPersonCreatedResponse};
use app_108jobs_utils::error::FastJobResult;

pub async fn list_person_created(
  data: Query<ListPersonCreated>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListPersonCreatedResponse>> {
  let limit = check_fetch_limit(data.limit)?;
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PostView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let post_views = PostView::list_created(
    &mut context.pool(),
    &local_user_view.person,
    data.language_id,
    cursor_data,
    data.page_back,
    Some(limit),
    None,
    data.post_kind,
    data.logistics_status,
  )
  .await?;

  let next_page = post_views.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = post_views.first().map(PaginationCursorBuilder::to_cursor);

  // User is viewing their own created posts, so they are the Employer
  let is_admin = local_user_view.local_user.admin;
  let viewer = if is_admin {
    LogisticsViewer::Admin
  } else {
    LogisticsViewer::Employer(local_user_view.person.id)
  };

  // Batch load logistics for all posts
  let created = load_logistics_for_post_views(post_views, &mut context.pool(), viewer, is_admin).await?;

  Ok(Json(ListPersonCreatedResponse {
    created,
    next_page,
    prev_page,
  }))
}
