use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::utils::check_category_deleted_removed;
use app_108jobs_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use app_108jobs_db_schema::{
  source::comment::{Comment, CommentUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_comment::{
  api::DeleteComment,
  CommentView,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

pub async fn delete_comment(
  data: Json<DeleteComment>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteComment>> {
  let comment_id = data.comment_id;
  let local_instance_id = local_user_view.person.instance_id;
  let orig_comment = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Dont delete it if its already been deleted.
  if orig_comment.comment.deleted == data.deleted {
    Err(FastJobErrorType::CouldntUpdateComment)?
  }

  check_category_deleted_removed(&orig_comment.category)?;

  // Verify that only the creator can delete
  if local_user_view.person.id != orig_comment.creator.id {
    Err(FastJobErrorType::NoCommentEditAllowed)?
  }

  // Do the delete
  let deleted = data.deleted;
  let updated_comment = Comment::update(
    &mut context.pool(),
    comment_id,
    &CommentUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await?;
  
  ActivityChannel::submit_activity(
    SendActivityData::DeleteComment(
      updated_comment,
      local_user_view.person.clone(),
      orig_comment.category,
    ),
    &context,
  )?;

  Ok(Json(
    DeleteComment {
      comment_id,
      deleted: true,
    }
  ))
}
