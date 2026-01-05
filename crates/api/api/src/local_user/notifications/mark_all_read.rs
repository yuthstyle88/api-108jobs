use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::{
  comment_reply::CommentReply,
  person_comment_mention::PersonCommentMention,
  person_post_mention::PersonPostMention,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::FastJobResult;

pub async fn mark_all_notifications_read(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let person_id = local_user_view.person.id;

  // Mark all comment_replies as read
  CommentReply::mark_all_as_read(&mut context.pool(), person_id).await?;

  // Mark all comment mentions as read
  PersonCommentMention::mark_all_as_read(&mut context.pool(), person_id).await?;

  // Mark all post mentions as read
  PersonPostMention::mark_all_as_read(&mut context.pool(), person_id).await?;
  
  Ok(Json(SuccessResponse::default()))
}
