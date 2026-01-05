use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::post::PostActions;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::PostView;
use app_108jobs_db_views_site::api::{ListPersonHidden, ListPersonHiddenResponse};
use app_108jobs_utils::error::FastJobResult;

pub async fn list_person_hidden(
  data: Query<ListPersonHidden>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListPersonHiddenResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PostActions::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let hidden = PostView::list_hidden(
    &mut context.pool(),
    &local_user_view.person,
    cursor_data,
    data.page_back,
    data.limit,
    None,
  )
  .await?;

  let next_page = hidden.last().map(PostView::to_post_actions_cursor);
  let prev_page = hidden.first().map(PostView::to_post_actions_cursor);

  Ok(Json(ListPersonHiddenResponse {
    hidden,
    next_page,
    prev_page,
  }))
}
