use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use lemmy_api_utils::{claims::Claims, context::FastJobContext};
use lemmy_db_schema::source::{
  email_verification::EmailVerification,
  local_user::{LocalUser, LocalUserUpdateForm},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{VerifyEmail, VerifyEmailSuccessResponse};
use lemmy_email::account::send_email_verified_email;
use lemmy_utils::error::FastJobResult;

pub async fn verify_email(
  data: Json<VerifyEmail>,
  req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<VerifyEmailSuccessResponse>> {
  let code = data.code.clone();
  let verification = EmailVerification::read_for_code(&mut context.pool(), &code).await?;
  let local_user_id = verification.local_user_id;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;

  let form = LocalUserUpdateForm {
    // necessary in case this is a new signup
    email_verified: Some(true),
    accepted_application: Some(false),
    // necessary in case multilang of an existing user was changed
    email: Some(Some(verification.email)),
    ..Default::default()
  };

  LocalUser::update(&mut context.pool(), local_user_id, &form).await?;

  EmailVerification::delete_old_codes_for_local_user(&mut context.pool(), local_user_id).await?;

  send_email_verified_email(&local_user_view, context.settings()).await?;
  let jwt = Claims::generate(
    local_user_view.local_user.id,
    local_user_view.local_user.email,
    local_user_view.local_user.interface_language,
    local_user_view.local_user.accepted_application,
    req,
    &context,
  )
  .await?;

  Ok(Json(VerifyEmailSuccessResponse { jwt }))
}
