use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_db_views_inbox_combined::api::GetUnreadRegistrationApplicationCountResponse;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_registration_applications::RegistrationApplicationView;
use app_108jobs_utils::error::FastJobResult;

pub async fn get_unread_registration_application_count(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetUnreadRegistrationApplicationCountResponse>> {
  let local_site = context.site_config().get().await?.site_view.local_site;

  // Only let admins do this
  is_admin(&local_user_view)?;

  let verified_email_only = local_site.require_email_verification;

  let registration_applications =
    RegistrationApplicationView::get_unread_count(&mut context.pool(), verified_email_only).await?;

  Ok(Json(GetUnreadRegistrationApplicationCountResponse {
    registration_applications,
  }))
}
