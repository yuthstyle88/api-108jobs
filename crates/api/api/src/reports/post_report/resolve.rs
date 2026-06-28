use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db::{source::post_report::PostReport, traits::Reportable};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_reports::{
  api::{PostReportResponse, ResolvePostReport},
  PostReportView,
};
use app_108jobs_core::error::FastJobResult;

/// Resolves or unresolves a post report and notifies the moderators of the category
pub async fn resolve_post_report(
  data: Json<ResolvePostReport>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PostReportResponse>> {
  let report_id = data.report_id;
  let person_id = local_user_view.person.id;

  if data.resolved {
    PostReport::resolve(&mut context.pool(), report_id, person_id).await?;
  } else {
    PostReport::unresolve(&mut context.pool(), report_id, person_id).await?;
  }

  let post_report_view = PostReportView::read(&mut context.pool(), report_id, person_id).await?;

  Ok(Json(PostReportResponse { post_report_view }))
}
