use crate::check_report_reason;
use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{
  context::FastJobContext,
  utils::{check_proposal_deleted_or_removed, slur_regex},
};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  source::proposal_report::{ProposalReport, ProposalReportForm},
  traits::Reportable,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::ProposalView;
use app_108jobs_db_views_reports::{
  api::{CreateProposalReportRequest, ProposalReportResponse},
  ProposalReportView,
};

/// Creates a comment report and notifies the moderators of the category
pub async fn create_comment_report(
  data: Json<CreateProposalReportRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProposalReportResponse>> {
  let reason = data.reason.trim().to_string();
  let slur_regex = slur_regex(&context).await?;
  check_report_reason(&reason, &slur_regex)?;

  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let comment_id = data.proposal_id;
  let comment_view = ProposalView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Don't allow creating reports for removed / deleted proposals
  check_proposal_deleted_or_removed(&comment_view.proposal)?;

  let report_form = ProposalReportForm {
    creator_id: person_id,
    comment_id,
    original_comment_text: comment_view.proposal.content,
    reason,
    violates_instance_rules: data.violates_instance_rules.unwrap_or_default(),
  };

  let report = ProposalReport::report(&mut context.pool(), &report_form).await?;

  let proposal_report_view =
    ProposalReportView::read(&mut context.pool(), report.id, person_id).await?;

  Ok(Json(ProposalReportResponse {
    proposal_report_view,
  }))
}
