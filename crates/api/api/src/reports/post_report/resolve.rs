use actix_web::web::Data;
use either::Either;
use app_108jobs_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use app_108jobs_db_schema::{source::post_report::PostReport, traits::Reportable};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_reports::{
  api::{PostReportResponse, ResolvePostReport},
  PostReportView,
};
use app_108jobs_utils::error::FastJobResult;

/// Resolves or unresolves a post report and notifies the moderators of the category
pub async fn resolve_post_report(
  data: Json<ResolvePostReport>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostReportResponse>> {
  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  let report = PostReportView::read(&mut context.pool(), report_id, person_id).await?;

  let person_id = local_user_view.person.id;

  if data.resolved {
    PostReport::resolve(&mut context.pool(), report_id, person_id).await?;
  } else {
    PostReport::unresolve(&mut context.pool(), report_id, person_id).await?;
  }

  let post_report_view = PostReportView::read(&mut context.pool(), report_id, person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      actor: local_user_view.person,
      report_creator: report.creator,
      receiver: Either::Right(post_report_view.category.clone()),
    },
    &context,
  )?;

  Ok(Json(PostReportResponse { post_report_view }))
}
