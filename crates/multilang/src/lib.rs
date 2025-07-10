// Avoid warnings for unused 0.19 website
#![allow(dead_code)]

use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{FastJobErrorExt, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
};
use lettre::{
  message::{Mailbox, MultiPart},
  transport::smtp::extension::ClientId,
  Address,
  AsyncTransport,
  Message,
};
use std::{fs, str::FromStr, sync::OnceLock};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use crate::loader::Lang;
use crate::namespace::{AllTranslations, NamespaceTranslations};

pub mod account;
pub mod notifications;
pub mod loader;
pub mod namespace;

type AsyncSmtpTransport = lettre::AsyncSmtpTransport<lettre::Tokio1Executor>;

fn inbox_link(settings: &Settings) -> String {
  format!("{}/inbox", settings.get_protocol_and_hostname())
}

async fn send_email(
  subject: &str,
  to_email: &str,
  to_username: &str,
  html: &str,
  settings: &Settings,
) -> FastJobResult<()> {
  static MAILER: OnceLock<AsyncSmtpTransport> = OnceLock::new();
  let email_config = settings.email.clone().ok_or(FastJobErrorType::NoEmailSetup)?;

  #[expect(clippy::expect_used)]
  let mailer = MAILER.get_or_init(|| {
    AsyncSmtpTransport::from_url(&email_config.connection)
      .expect("init multilang transport")
      .hello_name(ClientId::Domain(settings.hostname.clone()))
      .build()
  });

  // use usize::MAX as the line wrap length, since lettre handles the wrapping for us
  let plain_text = html2text::from_read(html.as_bytes(), usize::MAX)?;

  let smtp_from_address = &email_config.smtp_from_address;

  let email = Message::builder()
    .from(
      smtp_from_address
        .parse()
        .with_fastjob_type(FastJobErrorType::InvalidEmailAddress(
          smtp_from_address.into(),
        ))?,
    )
    .to(Mailbox::new(
      Some(to_username.to_string()),
      Address::from_str(to_email)
        .with_fastjob_type(FastJobErrorType::InvalidEmailAddress(to_email.into()))?,
    ))
    .message_id(Some(format!("<{}@{}>", Uuid::new_v4(), settings.hostname)))
    .subject(subject)
    .multipart(MultiPart::alternative_plain_html(
      plain_text,
      html.to_string(),
    ))
    .with_fastjob_type(FastJobErrorType::EmailSendFailed)?;

  mailer
    .send(email)
    .await
    .with_fastjob_type(FastJobErrorType::EmailSendFailed)?;

  Ok(())
}

#[allow(clippy::expect_used)]
fn user_language(local_user_view: &LocalUserView) -> Lang {
  let preferred_lang = &local_user_view.local_user.interface_language;

  Lang::from_str(&preferred_lang).unwrap_or_else(|| {
    tracing::warn!("Unsupported language '{}', falling back to default", preferred_lang);
    Lang::default()
  })
}

fn user_email(local_user_view: &LocalUserView) -> FastJobResult<SensitiveString> {
  local_user_view
    .local_user
    .email
    .clone()
    .ok_or(FastJobErrorType::EmailRequired.into())
}

pub fn load_all_translations(dir: &Path) -> FastJobResult<AllTranslations> {
  let mut all_translations = HashMap::new();

  for &lang in Lang::all() {
    let lang_dir = dir.join(lang.as_str());
    let mut namespaces = HashMap::new();

    if !lang_dir.exists() {
      continue;
    }

    for entry in fs::read_dir(&lang_dir)? {
      let entry = entry?;
      let path = entry.path();

      if is_json_file(&path) {
        let namespace = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name in {:?}", path))?
            .to_string();

        let content = fs::read_to_string(&path)?;
        let parsed: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse {:?}: {}", path, e))?;

        namespaces.insert(namespace, NamespaceTranslations(parsed));
      }
    }

    all_translations.insert(lang, namespaces);
  }

  Ok(all_translations)
}



fn is_json_file(path: &PathBuf) -> bool {
  path.extension().map_or(false, |ext| ext == "json")
}
