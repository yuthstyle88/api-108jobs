use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_db_schema::{source::category_report::CategoryReport, traits::Reportable};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_reports::{
  api::{CategoryReportResponse, ResolveCategoryReport},
  CategoryReportView,
};
use app_108jobs_utils::error::FastJobResult;

pub async fn resolve_category_report(
  data: Json<ResolveCategoryReport>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CategoryReportResponse>> {
  is_admin(&local_user_view)?;

  let report_id = data.report_id;
  let person_id = local_user_view.person.id;
  if data.resolved {
    CategoryReport::resolve(&mut context.pool(), report_id, person_id).await?;
  } else {
    CategoryReport::unresolve(&mut context.pool(), report_id, person_id).await?;
  }

  let category_report_view =
    CategoryReportView::read(&mut context.pool(), report_id, person_id).await?;

  Ok(Json(CategoryReportResponse {
    category_report_view,
  }))
}
