use actix_web::web::{Json};
use actix_web::web::Data;
use lemmy_api_utils::utils::check_community_deleted_removed;
use lemmy_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::comment::{Comment, CommentUpdateForm},
  traits::Crud,
};
use lemmy_db_views_comment::{
  api::{CommentResponse, DistinguishComment},
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

pub async fn distinguish_comment(
  data: Json<DistinguishComment>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommentResponse>> {
  let local_instance_id = local_user_view.person.instance_id;

  let orig_comment = CommentView::read(
    &mut context.pool(),
    data.comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Verify that only the creator can distinguish
  if local_user_view.person.id != orig_comment.creator.id {
    Err(FastJobErrorType::NoCommentEditAllowed)?
  }

  check_community_deleted_removed(&orig_comment.community)?;

  // Update the Comment
  let form = CommentUpdateForm {
    distinguished: Some(data.distinguished),
    ..Default::default()
  };

  let comment = Comment::update(&mut context.pool(), data.comment_id, &form).await?;
  ActivityChannel::submit_activity(SendActivityData::UpdateComment(comment), &context)?;

  let comment_view = CommentView::read(
    &mut context.pool(),
    data.comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  Ok(Json(CommentResponse { comment_view }))
}
