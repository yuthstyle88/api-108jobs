use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_private_instance};
use app_108jobs_db_schema::traits::PaginationCursorBuilder;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_person_saved_combined::{
  impls::PersonSavedCombinedQuery,
  ListPersonSaved,
  ListPersonSavedResponse,
  PersonSavedCombinedView,
};
use app_108jobs_utils::error::FastJobResult;
pub async fn list_person_saved(
  data: Query<ListPersonSaved>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListPersonSavedResponse>> {
  let site_view = context.site_config().get().await?.site_view;

  check_private_instance(&Some(local_user_view.clone()), &site_view.local_site)?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PersonSavedCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let saved = PersonSavedCombinedQuery {
    type_: data.type_,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
    no_limit: None,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  let next_page = saved.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = saved.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListPersonSavedResponse {
    saved,
    next_page,
    prev_page,
  }))
}
