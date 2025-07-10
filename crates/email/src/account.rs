use std::env;
use std::path::PathBuf;
use crate::{send_email, user_email, user_language};
use lemmy_db_schema::{
  source::{
    email_verification::{EmailVerification, EmailVerificationForm},
    local_site::LocalSite,
    password_reset_request::PasswordResetRequest,
  },
  utils::DbPool,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::FastJobResult,
  settings::structs::Settings,
  utils::markdown::markdown_to_html,
};
use lemmy_utils::error::{FastJobError, FastJobErrorType};

pub async fn send_password_reset_email(
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> FastJobResult<()> {
  // Generate a random token
  let token = uuid::Uuid::new_v4().to_string();

  let lang = user_language(user);
  let subject = &lang.password_reset_subject(&user.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let reset_link = format!("{}/password_change/{}", protocol_and_hostname, &token);
  let email = user_email(user)?;
  let body = &lang.password_reset_body(reset_link, &user.person.name);
  send_email(subject, &email, &user.person.name, body, settings).await?;

  // Insert the row after successful send, to avoid using daily reset limit while
  // email sending is broken.
  let local_user_id = user.local_user.id;
  PasswordResetRequest::create(pool, local_user_id, token.clone()).await?;
  Ok(())
}

/// Send a verification email
pub async fn send_verification_email(
  _local_site: &LocalSite,
  user: &LocalUserView,
  new_email: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> FastJobResult<()> {
  // Generate a 6-digit verification code
  let verification_code = {
    use rand::Rng;
    let mut rng = rand::rng();
    format!("{:06}", rng.random_range(0..1000000))
  };

  let form = EmailVerificationForm {
    local_user_id: user.local_user.id,
    email: new_email.to_string(),
    verification_token: verification_code.clone(),
  };
  EmailVerification::create(pool, &form).await?;

  // Read the HTML email template
  // Get the current working directory
  let cwd = env::current_dir()?;

  // Join with the relative path
  let template_path: PathBuf = cwd.join("crates/api/api_utils/src/templates/email_verification.html");
  let template = std::fs::read_to_string(template_path)?; // Replace the placeholders in the template
  println!("Template dir {}", template);
  let html_body = template
    .replace("{{ verification_code }}", &verification_code)
    .replace("{{ to_email }}", new_email);

  let lang = user_language(user);
  let subject = lang.verify_email_subject(&settings.hostname);

  send_email(&subject, new_email, &user.person.name, &html_body, settings).await
}

/// Returns true if email was sent.
pub async fn send_verification_email_if_required(
  local_site: &LocalSite,
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> FastJobResult<bool> {
  
  if user.local_user.email_verified {
    return Err(FastJobError::from(FastJobErrorType::EmailAlreadyVerified))
  }
  
  if !user.local_user.admin
    && local_site.require_email_verification
    && !user.local_user.email_verified
  {
    let email = user_email(user)?;
    send_verification_email(local_site, user, &email, pool, settings).await?;
    Ok(true)
  } else {
    Ok(false)
  }
}

pub async fn send_application_approved_email(
  user: &LocalUserView,
  settings: &Settings,
) -> FastJobResult<()> {
  let lang = user_language(user);
  let subject = lang.registration_approved_subject(&user.person.name);
  let email = user_email(user)?;
  let body = lang.registration_approved_body(&settings.hostname);
  send_email(&subject, &email, &user.person.name, &body, settings).await?;
  Ok(())
}

pub async fn send_application_denied_email(
  user: &LocalUserView,
  deny_reason: Option<String>,
  settings: &Settings,
) -> FastJobResult<()> {
  let lang = user_language(user);
  let subject = lang.registration_denied_subject(&user.person.name);
  let email = user_email(user)?;
  let body = match deny_reason {
    Some(deny_reason) => {
      let markdown = markdown_to_html(&deny_reason);
      lang.registration_denied_reason_body(&settings.hostname, &markdown)
    }
    None => lang.registration_denied_body(&settings.hostname),
  };
  send_email(&subject, &email, &user.person.name, &body, settings).await?;
  Ok(())
}

pub async fn send_email_verified_email(
  user: &LocalUserView,
  settings: &Settings,
) -> FastJobResult<()> {
  let lang = user_language(user);
  let subject = lang.email_verified_subject(&user.person.name);
  let email = user_email(user)?;
  let body = lang.email_verified_body();
  send_email(&subject, &email, &user.person.name, body, settings).await?;
  Ok(())
}
