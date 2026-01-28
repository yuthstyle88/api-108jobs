use actix_web::web::Data;
use actix_web::web::Json;
use either::Either;
use app_108jobs_api_utils::{
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
};
use app_108jobs_db_schema::{
  source::{category_report::CategoryReport, site::Site},
  traits::Reportable,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_reports::{
  api::{CategoryReportResponse, ResolveCategoryReport},
  CategoryReportView,
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};

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
  let category = category_report_view
    .category
    .as_ref()
    .ok_or(FastJobErrorType::NotFound)?;
  let site = Site::read_from_instance_id(
    &mut context.pool(),
    category.instance_id,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::SendResolveReport {
      actor: local_user_view.person,
      report_creator: category_report_view.creator.clone(),
      receiver: Either::Left(site),
    },
    &context,
  )?;

  Ok(Json(CategoryReportResponse {
    category_report_view,
  }))
}
