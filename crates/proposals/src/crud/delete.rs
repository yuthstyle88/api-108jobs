use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_category_deleted_removed};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  source::proposal::{Proposal, ProposalUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::{
  api::{DeleteComment, DeleteCommentRequest},
  ProposalView,
};

pub async fn delete_comment(
  data: Json<DeleteCommentRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<DeleteComment>> {
  let comment_id = data.comment_id;
  let local_instance_id = local_user_view.person.instance_id;
  let orig_comment = ProposalView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Dont delete it if its already been deleted.
  if orig_comment.proposal.deleted == data.deleted {
    Err(FastJobErrorType::CouldntUpdateComment)?
  }

  check_category_deleted_removed(
    orig_comment
      .category
      .as_ref()
      .ok_or(FastJobErrorType::NotFound)?,
  )?;

  // Verify that only the creator can delete
  if local_user_view.person.id != orig_comment.creator.id {
    Err(FastJobErrorType::NoCommentEditAllowed)?
  }

  // Do the delete
  let deleted = data.deleted;
  Proposal::update(
    &mut context.pool(),
    comment_id,
    &ProposalUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await?;

  Ok(Json(DeleteComment {
    comment_id,
    deleted: true,
  }))
}
