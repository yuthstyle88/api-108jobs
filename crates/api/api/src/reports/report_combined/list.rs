use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::check_local_user_valid;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_report_combined::{
  api::{ListReports, ListReportsResponse},
  impls::ReportCombinedQuery,
  ReportCombinedView,
};
use lemmy_utils::error::FastJobResult;

/// Lists reports for a category if an id is supplied
/// or returns all reports for communities a user moderates
pub async fn list_reports(
  data: Query<ListReports>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListReportsResponse>> {
  let my_reports_only = data.my_reports_only;

  // Only check mod or admin status when not viewing my reports
  if !my_reports_only.unwrap_or_default() {
    check_local_user_valid(&local_user_view)?;
  }

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(ReportCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let reports = ReportCombinedQuery {
    category_id: data.category_id,
    post_id: data.post_id,
    type_: data.type_,
    unresolved_only: data.unresolved_only,
    show_category_rule_violations: data.show_category_rule_violations,
    my_reports_only,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  let next_page = reports.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = reports.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListReportsResponse {
    reports,
    next_page,
    prev_page,
  }))
}
