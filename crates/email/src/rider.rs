use crate::{send_email, user_email, user_language};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::{
  error::FastJobResult, settings::structs::Settings, utils::markdown::markdown_to_html,
};

pub async fn send_rider_application_approved_email(
  user: &LocalUserView,
  settings: &Settings,
) -> FastJobResult<()> {
  let lang = user_language(user);
  let subject = lang.rider_application_approved_subject(&user.person.name);
  let email = user_email(user)?;

  let footer = (
    lang.contact_us(),
    lang.copyright(),
    lang.privacy_policy(),
    lang.regards(),
    lang.sent_to(&*email),
    lang.team(),
    lang.terms_of_service(),
  );

  let title = lang.rider_approved_title(&user.person.name);
  let message = lang.rider_approved_message(&settings.hostname);
  let next_steps = lang.rider_approved_next_steps();

  let body = lang.rider_application_approved_email(
    footer.0,
    footer.1,
    message,
    next_steps,
    footer.2,
    footer.3,
    footer.4,
    footer.5,
    footer.6,
    title,
  );

  send_email(&subject, &email, &user.person.name, &body, settings).await?;
  Ok(())
}

pub async fn send_rider_application_denied_email(
  user: &LocalUserView,
  deny_reason: Option<String>,
  settings: &Settings,
) -> FastJobResult<()> {
  let lang = user_language(user);
  let subject = lang.rider_application_denied_subject(&user.person.name);
  let email = user_email(user)?;

  let footer = (
    lang.contact_us(),
    lang.copyright(),
    lang.privacy_policy(),
    lang.regards(),
    lang.sent_to(&*email),
    lang.team(),
    lang.terms_of_service(),
  );

  let title = lang.rider_denied_title();
  let message = lang.rider_denied_message(&settings.hostname);
  let contact = lang.rider_denied_contact();

  let body = match deny_reason {
    Some(deny_reason) => {
      let markdown = markdown_to_html(&deny_reason);
      let reason_title = lang.rider_denied_reason_title();
      lang.rider_application_denied_reason_email(
        footer.0,
        contact,
        footer.1,
        message,
        footer.2,
        reason_title,
        markdown,
        footer.3,
        footer.4,
        footer.5,
        footer.6,
        title,
      )
    }
    None => lang.rider_application_denied_email(
      footer.0,
      contact,
      footer.1,
      message,
      footer.2,
      footer.3,
      footer.4,
      footer.5,
      footer.6,
      title,
    ),
  };

  send_email(&subject, &email, &user.person.name, &body, settings).await?;
  Ok(())
}
