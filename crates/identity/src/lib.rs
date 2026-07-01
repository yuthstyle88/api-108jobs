use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_db_views_local_user::LocalUserView;
use totp_rs::{Secret, TOTP};

pub mod change_password;
pub mod change_password_after_reset;
pub mod generate_totp_secret;
pub mod identity_card;
pub mod list_logins;
pub mod login;
pub mod logout;
pub mod refresh;
pub mod resend_verification_email;
pub mod reset_password;
pub mod update_totp;
pub mod validate_auth;
pub mod verify_email;

pub(crate) fn check_totp_2fa_valid(
  local_user_view: &LocalUserView,
  totp_token: &Option<String>,
  site_name: &str,
) -> FastJobResult<()> {
  // Throw an error if their token is missing
  let token = totp_token
    .as_deref()
    .ok_or(FastJobErrorType::MissingTotpToken)?;
  let secret = local_user_view
    .local_user
    .totp_2fa_secret
    .as_deref()
    .ok_or(FastJobErrorType::MissingTotpSecret)?;

  let totp = build_totp_2fa(site_name, &local_user_view.person.name, secret)?;

  let check_passed = totp.check_current(token)?;
  if !check_passed {
    return Err(FastJobErrorType::IncorrectTotpToken.into());
  }

  Ok(())
}

pub(crate) fn generate_totp_2fa_secret() -> String {
  Secret::generate_secret().to_string()
}

pub(crate) fn build_totp_2fa(hostname: &str, username: &str, secret: &str) -> FastJobResult<TOTP> {
  let sec = Secret::Raw(secret.as_bytes().to_vec());
  let sec_bytes = sec
    .to_bytes()
    .with_fastjob_type(FastJobErrorType::CouldntParseTotpSecret)?;

  TOTP::new(
    totp_rs::Algorithm::SHA1,
    6,
    1,
    30,
    sec_bytes,
    Some(hostname.to_string()),
    username.to_string(),
  )
  .with_fastjob_type(FastJobErrorType::CouldntGenerateTotp)
}
