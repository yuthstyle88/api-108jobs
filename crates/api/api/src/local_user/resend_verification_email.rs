use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::{ResendVerificationEmail, SuccessResponse};
use app_108jobs_email::account::send_verification_email_if_required;
use app_108jobs_utils::error::FastJobResult;

pub async fn resend_verification_email(
  data: Json<ResendVerificationEmail>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<SuccessResponse>> {
  let site_view = context.site_config().get().await?.site_view;
  let email = data.email.to_string();

  // Fetch that multilang
  let local_user_view = LocalUserView::find_by_email(&mut context.pool(), &email).await?;

  send_verification_email_if_required(
    &site_view.local_site,
    &local_user_view,
    &mut context.pool(),
    context.settings(),
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
