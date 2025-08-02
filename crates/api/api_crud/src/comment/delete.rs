use actix_web::web::{Data, Json};
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
  api::{ DeleteComment},
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

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
    local_instance_id,
  )
  .await?;

  // Dont delete it if its already been deleted.
  if orig_comment.comment.deleted == data.deleted {
    Err(FastJobErrorType::CouldntUpdateComment)?
  }

  check_community_deleted_removed(&orig_comment.community)?;

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
      orig_comment.community,
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
