use crate::check_totp_2fa_valid;
use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use bcrypt::verify;
use lemmy_api_utils::{
  claims::Claims,
  context::FastJobContext,
  utils::{check_email_verified, check_local_user_deleted, check_registration_application},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::LoginRequest;
use lemmy_db_views_site::api::{Login, LoginResponse};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

pub async fn login(
  data: Json<LoginRequest>,
  req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<LoginResponse>> {
  let data: Login = data.0.clone().try_into()?;

  let site_view = context.site_config().get().await?.site_view;

  // Fetch that username / multilang
  let username_or_email = data.username_or_email.clone();
  let local_user_view =
    LocalUserView::find_by_email_or_name(&mut context.pool(), &username_or_email).await?;

  // Verify the password
  let valid: bool = local_user_view
    .local_user
    .password_encrypted
    .as_ref()
    .and_then(|password_encrypted| verify(&data.password, password_encrypted).ok())
    .unwrap_or(false);
  if !valid {
    Err(FastJobErrorType::IncorrectLogin)?
  }
  check_local_user_deleted(&local_user_view)?;
  check_email_verified(&local_user_view, &site_view)?;

  check_registration_application(&local_user_view, &site_view.local_site, &mut context.pool())
    .await?;

  // Check the totp if enabled
  if local_user_view.local_user.totp_2fa_enabled {
    check_totp_2fa_valid(
      &local_user_view,
      &data.totp_2fa_token,
      &context.settings().hostname,
    )?;
  }
  let jwt = Claims::generate(
    local_user_view.local_user.id,
    local_user_view.local_user.email,
    local_user_view.local_user.interface_language,
    local_user_view.local_user.accepted_application,
    req,
    &context,
  )
  .await?;

  Ok(Json(LoginResponse {
    jwt: Some(jwt.clone()),
    verify_email_sent: false,
    registration_created: false,
    application_pending: false,
  }))
}
