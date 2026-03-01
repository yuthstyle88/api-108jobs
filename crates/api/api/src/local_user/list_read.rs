use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_fetch_limit};
use app_108jobs_db_schema::source::post::PostActions;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::{
  logistics::{load_logistics_for_post_views, LogisticsViewer},
  PostView,
};
use app_108jobs_db_views_site::api::{ListPersonRead, ListPersonReadResponse};
use app_108jobs_utils::error::FastJobResult;

pub async fn list_person_read(
  data: Query<ListPersonRead>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListPersonReadResponse>> {
  let limit = check_fetch_limit(data.limit)?;
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PostActions::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let post_views = PostView::list_read(
    &mut context.pool(),
    &local_user_view.person,
    cursor_data,
    data.page_back,
    Some(limit),
    None,
  )
  .await?;

  let next_page = post_views.last().map(PostView::to_post_actions_cursor);
  let prev_page = post_views.first().map(PostView::to_post_actions_cursor);

  // Determine viewer role for logistics projection
  let is_admin = local_user_view.local_user.admin;
  let viewer = if is_admin {
    LogisticsViewer::Admin
  } else {
    LogisticsViewer::Public // User is viewing read posts, not their own created posts
  };

  // Batch load logistics for all posts
  let read = load_logistics_for_post_views(post_views, &mut context.pool(), viewer, is_admin).await?;

  Ok(Json(ListPersonReadResponse {
    read,
    next_page,
    prev_page,
  }))
}
