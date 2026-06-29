use crate::check_report_reason;
use actix_web::web::{Data, Json};
use app_108jobs_api_utils::{context::FastJobContext, utils::slur_regex};
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::{
  source::{
    category::Category,
    category_report::{CategoryReport, CategoryReportForm},
  },
  traits::{Crud, Reportable},
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_reports::{
  api::{CategoryReportResponse, CreateCategoryReportRequest},
  CategoryReportView,
};

pub async fn create_category_report(
  data: Json<CreateCategoryReportRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CategoryReportResponse>> {
  let reason = data.reason.trim().to_string();
  let slur_regex = slur_regex(&context).await?;
  check_report_reason(&reason, &slur_regex)?;

  let person_id = local_user_view.person.id;
  let category_id = data.category_id;
  let category = Category::read(&mut context.pool(), category_id).await?;

  let report_form = CategoryReportForm {
    creator_id: person_id,
    category_id,
    original_category_banner: category.banner,
    original_category_description: category.description,
    original_category_icon: category.icon,
    original_category_name: category.name,
    original_category_sidebar: category.sidebar,
    original_category_title: category.title,
    reason,
  };

  let report = CategoryReport::report(&mut context.pool(), &report_form).await?;

  let category_report_view =
    CategoryReportView::read(&mut context.pool(), report.id, person_id).await?;

  Ok(Json(CategoryReportResponse {
    category_report_view,
  }))
}
