use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_category_deleted_removed};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{source::proposal_report::ProposalReport, traits::Reportable};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_reports::{
  api::{ProposalReportResponse, ResolveProposalReport},
  ProposalReportView,
};

/// Resolves or unresolves a comment report and notifies the moderators of the category
pub async fn resolve_comment_report(
  data: Json<ResolveProposalReport>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ProposalReportResponse>> {
  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  let report = ProposalReportView::read(&mut context.pool(), report_id, person_id).await?;

  let person_id = local_user_view.person.id;
  check_category_deleted_removed(report.category.as_ref().ok_or(FastJobErrorType::NotFound)?)?;

  if data.resolved {
    ProposalReport::resolve(&mut context.pool(), report_id, person_id).await?;
  } else {
    ProposalReport::unresolve(&mut context.pool(), report_id, person_id).await?;
  }

  let report_id = data.report_id;
  let proposal_report_view =
    ProposalReportView::read(&mut context.pool(), report_id, person_id).await?;

  Ok(Json(ProposalReportResponse {
    proposal_report_view,
  }))
}
