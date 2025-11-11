use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use lemmy_api_utils::{claims::Claims, context::FastJobContext, utils::slur_regex};
use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_db_schema_file::enums::RegistrationMode;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_registration_applications::api::OAuthUserUpdateRequest;
use lemmy_db_views_site::api::LoginResponse;
use lemmy_email::admin::send_new_applicant_email_to_admins;
use lemmy_utils::utils::validation::password_length_check;
use lemmy_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::slurs::check_slurs,
};

pub async fn update_term(
  data: Json<OAuthUserUpdateRequest>,
  req: HttpRequest,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<LoginResponse>> {
  let pool = &mut context.pool();
  let data = data.into_inner();
  let site_view = context.site_config().get().await?.site_view;
  let local_site = site_view.local_site.clone();

  if local_site.registration_mode == RegistrationMode::Closed {
    Err(FastJobErrorType::RegistrationClosed)?
  }
  if let Some(pwd) = &data.password {
    password_length_check(&pwd)?;
  }

  if local_site.require_email_verification && data.email.is_none() {
    Err(FastJobErrorType::EmailRequired)?
  }

  let username = data.email.clone().unwrap();

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&username, &slur_regex)?;

  // Wrap the insert person, insert local user, and create registration,
  // in a transaction, so that if any fail, the rows aren't created.

  // Email the admins, only if email verification is not required
  if local_site.application_email_admins && !local_site.require_email_verification {
    send_new_applicant_email_to_admins(&username, pool, context.settings()).await?;
  }

  let mut login_response = LoginResponse {
    jwt: None,
    registration_created: false,
    verify_email_sent: false,
    accepted_terms: false,
  };
  // Log the user in directly if the site is not setup, or email verification and application aren't
  // required
  if !local_site.site_setup
    || (local_user_view.local_user.email_verified
      && !local_user_view.local_user.accepted_terms)
  {
    if data.password != data.password_verify {
      Err(FastJobErrorType::PasswordsDoNotMatch)?
    }

    // Update the user with the new password
    let password = data.password.clone();
    let user = LocalUser::update_term(
      &mut context.pool(),
      local_user_view.local_user.id,
      data.terms_accepted.unwrap(),
      &password.unwrap(),
    )
    .await?;

    let jwt = Claims::generate(user.id, user.email, user.interface_language, user.accepted_terms, user.admin, req, &context).await?;
    login_response.jwt = Some(jwt);
  }

  Ok(Json(login_response))
}
