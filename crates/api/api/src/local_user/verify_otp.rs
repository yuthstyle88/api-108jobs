use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use lemmy_api_utils::{claims::Claims, context::FastJobContext};
use lemmy_db_schema::source::{
  local_user::{LocalUser, LocalUserUpdateForm},
};
use lemmy_db_schema::source::otp_verification::OTPVerification;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  api::{VerifyEmailSuccessResponse},
  SiteView,
};
use lemmy_db_views_site::api::VerifyOTP;
use lemmy_email::account::send_email_verified_email;
use lemmy_utils::error::FastJobResult;

pub async fn verify_otp(
  data: Json<VerifyOTP>,
  req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<VerifyEmailSuccessResponse>> {
  SiteView::read_local(&mut context.pool()).await?;
  let otp = data.otp.clone();
  let verification = OTPVerification::read_for_otp(&mut context.pool(), &otp).await?;
  let local_user_id = verification.local_user_id;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;

  let form = LocalUserUpdateForm {
    // necessary in case this is a new signup
    email_verified: Some(true),
    accepted_application: Some(false),
    email: Some(Some(verification.email)),
    ..Default::default()
  };

  LocalUser::update(&mut context.pool(), local_user_id, &form).await?;

  OTPVerification::delete_old_otps_for_local_user(&mut context.pool(), local_user_id).await?;

  send_email_verified_email(&local_user_view, context.settings()).await?;
  let jwt = Claims::generate(
    local_user_view.local_user.id,
    local_user_view.local_user.email,
    local_user_view.local_user.role,
    local_user_view.local_user.interface_language,
    local_user_view.local_user.accepted_application,
    req,
    &context,
  )
  .await?;

  Ok(Json(VerifyEmailSuccessResponse { jwt }))
}
