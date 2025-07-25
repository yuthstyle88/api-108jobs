use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_comment::{
  api::{ListCommentLikes, ListCommentLikesResponse},
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_vote::VoteView;
use lemmy_utils::error::FastJobResult;

/// Lists likes for a comment
pub async fn list_comment_likes(
  data: Query<ListCommentLikes>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListCommentLikesResponse>> {
  let local_instance_id = local_user_view.person.instance_id;

  let comment_view = CommentView::read(
    &mut context.pool(),
    data.comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

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
