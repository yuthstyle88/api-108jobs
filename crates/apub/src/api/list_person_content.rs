use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_private_instance};
use app_108jobs_db_schema::traits::PaginationCursorBuilder;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_person_content_combined::{
  impls::PersonContentCombinedQuery, ListPersonContent, ListPersonContentResponse,
  PersonContentCombinedView,
};
use app_108jobs_utils::error::FastJobResult;

pub async fn list_person_content(
  data: Query<ListPersonContent>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<ListPersonContentResponse>> {
  let site_view = context.site_config().get().await?.site_view;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  let person_details_id = data.person_id.unwrap();

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PersonContentCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let content = PersonContentCombinedQuery {
    creator_id: person_details_id,
    type_: data.type_,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
    no_limit: None,
  }
  .list(
    &mut context.pool(),
    local_user_view.as_ref(),
    local_instance_id,
  )
  .await?;

  let next_page = content.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = content.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListPersonContentResponse {
    content,
    next_page,
    prev_page,
  }))
}
