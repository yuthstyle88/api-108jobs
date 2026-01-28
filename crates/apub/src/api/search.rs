use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::{check_conflicting_like_filters, check_private_instance};
use app_108jobs_db_schema::traits::PaginationCursorBuilder;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_search_combined::{
  impls::SearchCombinedQuery, Search, SearchCombinedView, SearchResponse,
};
use app_108jobs_utils::error::FastJobResult;

pub async fn search(
  data: Query<Search>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<SearchResponse>> {
  let site_view = context.site_config().get().await?.site_view;
  let local_site = site_view.local_site;
  let language_id = data.language_id;

  check_private_instance(&local_user_view, &local_site)?;
  check_conflicting_like_filters(data.liked_only, data.disliked_only)?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(SearchCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let pool = &mut context.pool();
  let search_query = SearchCombinedQuery {
    search_term: data.q.clone(),
    category_id: data.category_id,
    type_: data.r#type,
    language_id,
    creator_id: data.creator_id,
    sort: data.sort,
    time_range_seconds: data.time_range_seconds,
    intended_use: data.intended_use,
    job_type: data.job_type,
    budget_min: data.budget_min,
    budget_max: data.budget_max,
    requires_english: data.requires_english,
    post_kind: data.post_kind,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  };

  let results = search_query
    .list(pool, &local_user_view, &site_view.site)
    .await?;

  let next_page = results.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = results.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(SearchResponse {
    results,
    next_page,
    prev_page,
  }))
}
