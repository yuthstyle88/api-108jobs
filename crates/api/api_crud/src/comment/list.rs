use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{
  context::FastJobContext,
  utils::check_private_instance,
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_comment::{
  api::{GetComments, GetCommentsResponse},
  impls::CommentQuery,
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::FastJobResult;

pub async fn list_comments(
  data: Query<GetComments>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetCommentsResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = &site_view.local_site;
  let _local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, local_site)?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(CommentView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  tracing::info!("Comment list query params: post_id={:?}, limit={:?}", 
                 data.post_id, data.limit);

  let comments = CommentQuery {
    listing_type: data.type_,
    sort: data.sort,
    time_range_seconds: data.time_range_seconds,
    post_id: data.post_id,
    max_depth: data.max_depth,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  tracing::info!("Comment list query returned {} comments", comments.len());

  let next_page = comments
    .last()
    .map(PaginationCursorBuilder::to_cursor)
    .filter(|_| comments.len() as i64 == data.limit.unwrap_or(10));
  
  let prev_page = comments
    .first()
    .map(PaginationCursorBuilder::to_cursor)
    .filter(|_| data.page_back.unwrap_or_default());

  Ok(Json(GetCommentsResponse {
    comments,
    next_page,
    prev_page,
  }))
}