use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{build_response::build_proposal_response, context::FastJobContext};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  source::{
    local_user::LocalUser,
    mod_log::moderator::{ModRemoveProposal, ModRemoveProposalForm},
    proposal::{Proposal, ProposalUpdateForm},
    proposal_report::ProposalReport,
  },
  traits::{Crud, Reportable},
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::{
  api::{ProposalResponse, RemoveComment},
  ProposalView,
};

pub async fn remove_comment(
  data: Json<RemoveComment>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProposalResponse>> {
  let comment_id = data.proposal_id;
  let local_instance_id = local_user_view.person.instance_id;
  let orig_comment = ProposalView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  LocalUser::is_higher_mod_or_admin_check(
    &mut context.pool(),
    orig_comment
      .category
      .as_ref()
      .ok_or(FastJobErrorType::NotFound)?
      .id,
    local_user_view.person.id,
    vec![orig_comment.creator.id],
  )
  .await?;

  // Don't allow removing or restoring proposal which was deleted by user, as it would reveal
  // the proposal text in mod log.
  if orig_comment.proposal.deleted {
    return Err(FastJobErrorType::CouldntUpdateProposal.into());
  }

  // Do the remove
  let removed = data.removed;
  let updated_proposal = Proposal::update(
    &mut context.pool(),
    comment_id,
    &ProposalUpdateForm {
      removed: Some(removed),
      ..Default::default()
    },
  )
  .await?;

  ProposalReport::resolve_all_for_object(
    &mut context.pool(),
    comment_id,
    local_user_view.person.id,
  )
  .await?;

  // Mod tables
  let form = ModRemoveProposalForm {
    mod_person_id: local_user_view.person.id,
    comment_id: data.proposal_id,
    removed: Some(removed),
    reason: data.reason.clone(),
  };
  ModRemoveProposal::create(&mut context.pool(), &form).await?;

  let updated_comment_id = updated_proposal.id;

  Ok(Json(
    build_proposal_response(
      &context,
      updated_comment_id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}
