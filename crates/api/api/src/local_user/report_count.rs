use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_local_user_valid};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_report_combined::ReportCombinedViewInternal;
use app_108jobs_db_views_reports::api::{GetReportCount, GetReportCountResponse};

pub async fn report_count(
  data: Query<GetReportCount>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetReportCountResponse>> {
  check_local_user_valid(&local_user_view)?;

  let count = ReportCombinedViewInternal::get_report_count(
    &mut context.pool(),
    &local_user_view,
    data.category_id,
  )
  .await?;

  Ok(Json(GetReportCountResponse { count }))
}
