use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::is_admin;
use app_108jobs_db_views_comment::api::{ListCommentLikes, ListCommentLikesResponse};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_vote::VoteView;
use app_108jobs_utils::error::FastJobResult;

/// Lists likes for a comment
pub async fn list_comment_likes(
  data: Query<ListCommentLikes>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListCommentLikesResponse>> {
  is_admin(&local_user_view)?;
  
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(VoteView::from_comment_actions_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let comment_likes = VoteView::list_for_comment(
    &mut context.pool(),
    data.comment_id,
    cursor_data,
    data.page_back,
    data.limit,
  )
  .await?;

  let next_page = comment_likes
    .last()
    .map(VoteView::to_comment_actions_cursor);
  let prev_page = comment_likes
    .first()
    .map(VoteView::to_comment_actions_cursor);

  Ok(Json(ListCommentLikesResponse {
    comment_likes,
    next_page,
    prev_page,
  }))
}
