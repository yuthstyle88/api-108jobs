use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::is_admin};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_registration_applications::{
  api::{GetRegistrationApplication, RegistrationApplicationResponse},
  RegistrationApplicationView,
};
use app_108jobs_utils::error::FastJobResult;

/// Lists registration applications, filterable by undenied only.
pub async fn get_registration_application(
  data: Query<GetRegistrationApplication>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<RegistrationApplicationResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  // Read the view
  let registration_application =
    RegistrationApplicationView::read_by_person(&mut context.pool(), data.person_id).await?;

  Ok(Json(RegistrationApplicationResponse {
    registration_application,
  }))
}
