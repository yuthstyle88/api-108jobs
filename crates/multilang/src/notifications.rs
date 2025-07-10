use crate::{send_email};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  settings::structs::Settings,
};
use tracing::warn;

async fn send_email_to_user(
  local_user_view: &LocalUserView,
  subject: &str,
  body: &str,
  settings: &Settings,
) {
  if local_user_view.banned || !local_user_view.local_user.send_notifications_to_email {
    return;
  }

  if let Some(user_email) = &local_user_view.local_user.email {
    send_email(
      subject,
      user_email,
      &local_user_view.person.name,
      body,
      settings,
    )
    .await
    .unwrap_or_else(|e| warn!("{}", e));
  }
}
