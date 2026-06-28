use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  source::{
    local_user::LocalUser,
    mod_log::admin::{AdminPurgeProposal, AdminPurgeProposalForm},
    proposal::Proposal,
  },
  traits::Crud,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::{api::PurgeComment, ProposalView};
use app_108jobs_db_views_site::api::SuccessResponse;

pub async fn purge_comment(
  data: Json<PurgeComment>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  let comment_id = data.comment_id;
  let local_instance_id = local_user_view.person.instance_id;

  // Read the comment to get the post_id and category
  let comment_view = ProposalView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Also check that you're a higher admin
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![comment_view.creator.id],
  )
  .await?;

  let post_id = comment_view.proposal.post_id;

  // TODO read proposals for pictrs images and purge them

  Proposal::delete(&mut context.pool(), comment_id).await?;

  // Mod tables
  let form = AdminPurgeProposalForm {
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
    post_id,
  };
  AdminPurgeProposal::create(&mut context.pool(), &form).await?;

  Ok(Json(SuccessResponse::default()))
}
