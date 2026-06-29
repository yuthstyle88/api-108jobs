use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_category_deleted_removed};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  source::proposal::{Proposal, ProposalUpdateForm},
  traits::Crud,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::{
  api::{DistinguishComment, ProposalResponse},
  ProposalView,
};

pub async fn distinguish_comment(
  data: Json<DistinguishComment>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProposalResponse>> {
  let local_instance_id = local_user_view.person.instance_id;

  let orig_comment = ProposalView::read(
    &mut context.pool(),
    data.proposal_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Verify that only the creator can distinguish
  if local_user_view.person.id != orig_comment.creator.id {
    Err(FastJobErrorType::NoProposalEditAllowed)?
  }

  check_category_deleted_removed(
    orig_comment
      .category
      .as_ref()
      .ok_or(FastJobErrorType::NotFound)?,
  )?;

  // Update the Proposal
  let form = ProposalUpdateForm {
    distinguished: Some(data.distinguished),
    ..Default::default()
  };

  Proposal::update(&mut context.pool(), data.proposal_id, &form).await?;

  let proposal_view = ProposalView::read(
    &mut context.pool(),
    data.proposal_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  Ok(Json(ProposalResponse { proposal_view }))
}
