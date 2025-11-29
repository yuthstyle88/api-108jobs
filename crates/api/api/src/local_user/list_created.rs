use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::api::{ListPersonCreated, ListPersonCreatedResponse};
use lemmy_utils::error::FastJobResult;

pub async fn list_person_created(
  data: Query<ListPersonCreated>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListPersonCreatedResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PostView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let created = PostView::list_created(
    &mut context.pool(),
    &local_user_view.person,
    data.language_id,
    cursor_data,
    data.page_back,
    data.limit,
    None,
  )
  .await?;

  let next_page = created.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = created.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListPersonCreatedResponse {
    created,
    next_page,
    prev_page,
  }))
}
