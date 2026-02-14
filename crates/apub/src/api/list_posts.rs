use crate::api::{listing_type_with_default, post_sort_type_with_default};
use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_private_instance};
use app_108jobs_db_schema::traits::PaginationCursorBuilder;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_post::{
  api::{GetPosts, GetPostsResponse},
  impls::PostQuery,
  PostView,
};
use app_108jobs_utils::error::FastJobResult;

pub async fn list_posts(
  data: Query<GetPosts>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetPostsResponse>> {
  let site_view = context.site_config().get().await?.site_view;

  check_private_instance(&local_user_view, &site_view.local_site)?;

  let limit = data.limit;
  let category_id = data.category_id;
  let language_id = data.language_id;
  let show_hidden = data.show_hidden;
  // Show nsfw content if param is true, or if content_warning exists
  let hide_media = data.hide_media;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);
  let listing_type = Some(listing_type_with_default(
    data.type_,
    local_user,
    &site_view.local_site,
    category_id,
  ));

  let sort = Some(post_sort_type_with_default(
    data.sort,
    local_user,
    &site_view.local_site,
  ));

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PostView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let page_back = data.page_back;

  let posts = PostQuery {
    local_user,
    listing_type,
    language_id,
    sort,
    time_range_seconds: None,
    category_id,
    limit,
    show_hidden,
    show_read: None,
    self_promotion: data.self_promotion,
    hide_media,
    no_proposals_only: None,
    intended_use: data.intended_use,
    job_type: data.job_type,
    budget_min: data.budget_min,
    budget_max: data.budget_max,
    requires_english: data.requires_english,
    post_kind: data.post_kind,
    logistics_status: None,
    cursor_data,
    page_back,
  }
  .list(&site_view.site, &mut context.pool())
  .await?;
  // if this page wasn't empty, then there is a next page after the last post on this page
  let next_page = posts.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = posts.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(GetPostsResponse {
    posts,
    next_page,
    prev_page,
  }))
}
