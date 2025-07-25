use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{context::FastJobContext};
use lemmy_db_schema::{source::post::Post, traits::Crud};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{ListPostLikes, ListPostLikesResponse};
use lemmy_db_views_vote::VoteView;
use lemmy_utils::error::FastJobResult;

/// Lists likes for a post
pub async fn list_post_likes(
  data: Query<ListPostLikes>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListPostLikesResponse>> {
  let post = Post::read(&mut context.pool(), data.post_id).await?;

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(VoteView::from_post_actions_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let post_likes = VoteView::list_for_post(
    &mut context.pool(),
    data.post_id,
    cursor_data,
    data.page_back,
    data.limit,
  )
  .await?;

  let next_page = post_likes.last().map(VoteView::to_post_actions_cursor);
  let prev_page = post_likes.first().map(VoteView::to_post_actions_cursor);

  Ok(Json(ListPostLikesResponse {
    post_likes,
    next_page,
    prev_page,
  }))
}
