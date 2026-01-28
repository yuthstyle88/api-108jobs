use actix_web::web::Data;
use actix_web::web::Json;
use either::Either;
use app_108jobs_api_utils::utils::check_category_deleted_removed;
use app_108jobs_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use app_108jobs_db_schema::{source::comment_report::CommentReport, traits::Reportable};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_reports::{
  api::{CommentReportResponse, ResolveCommentReport},
  CommentReportView,
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

/// Resolves or unresolves a comment report and notifies the moderators of the category
pub async fn resolve_comment_report(
  data: Json<ResolveCommentReport>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CommentReportResponse>> {
  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  let report = CommentReportView::read(&mut context.pool(), report_id, person_id).await?;

  let person_id = local_user_view.person.id;
  check_category_deleted_removed(
    report
      .category
      .as_ref()
      .ok_or(FastJobErrorType::NotFound)?,
  )?;

  if data.resolved {
    CommentReport::resolve(&mut context.pool(), report_id, person_id).await?;
  } else {
    CommentReport::unresolve(&mut context.pool(), report_id, person_id).await?;
  }

  let report_id = data.report_id;
  let comment_report_view =
    CommentReportView::read(&mut context.pool(), report_id, person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      actor: local_user_view.person,
      report_creator: report.creator,
      receiver: Either::Right(comment_report_view.category.clone().ok_or(FastJobErrorType::NotFound)?),
    },
    &context,
  )?;

  Ok(Json(CommentReportResponse {
    comment_report_view,
  }))
}
