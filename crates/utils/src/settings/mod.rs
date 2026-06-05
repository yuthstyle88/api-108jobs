use crate::{error::FastJobResult, location_info};
use anyhow::{anyhow, Context};
use deser_hjson::from_str;
use regex::Regex;
use std::{env, fs, sync::LazyLock};
use structs::{PictrsConfig, Settings};
use url::Url;

pub mod secrets;
pub mod structs;

static DEFAULT_CONFIG_FILE: &str = "config/config.hjson";

#[allow(clippy::expect_used)]
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
  if env::var("app_108jobs_INITIALIZE_WITH_DEFAULT_SETTINGS").is_ok() {
    println!(
      "app_108jobs_INITIALIZE_WITH_DEFAULT_SETTINGS was set, any configuration file has been ignored."
    );
    println!("Use with other environment variables to configure this instance further; e.g. app_108jobs_DATABASE_URL.");
    Settings::default()
  } else {
    Settings::init().expect("Failed to load settings file, see documentation (https://join-app_108jobs.org/docs/en/administration/configuration.html).")
  }
});

#[allow(clippy::expect_used)]
static WEBFINGER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(
    "^acct:([a-zA-Z0-9_]{{3,}})@{}$",
    SETTINGS.hostname
  ))
  .expect("compile webfinger regex")
});

impl Settings {
  /// Reads config from configuration file and resolves all required secrets
  /// from environment variables. Fails fast if any required secret is missing
  /// or if the config file still contains a plaintext password.
  ///
  /// Note: The env var `app_108jobs_DATABASE_URL` is also consumed downstream
  /// in `app_108jobs_db_schema/src/lib.rs::get_database_url_from_env()`.
  /// Warning: Only call this once.
  pub(crate) fn init() -> FastJobResult<Self> {
    let path =
      env::var("app_108jobs_CONFIG_LOCATION").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string());
    let plain = fs::read_to_string(path)?;
    let mut config = from_str::<Settings>(&plain)?;
    if config.hostname == "unset" {
      return Err(anyhow!("Hostname variable is not set!").into());
    }
    config.apply_env_secrets(secrets::read_env)?;
    Ok(config)
  }

  /// Resolve required secrets from env vars (or a test reader) and merge them
  /// into this struct. Returns Err with a clear message if anything required
  /// is missing or malformed.
  pub fn apply_env_secrets<F>(&mut self, reader: F) -> FastJobResult<()>
  where
    F: Fn(&str) -> Option<String> + Copy,
  {
    let req = secrets::SecretRequirements {
      database_connection_in_file: self.database.connection_raw(),
      email_section_present: self
        .email
        .as_ref()
        .map(|e| !e.smtp_from_address.trim().is_empty())
        .unwrap_or(false),
      setup_section_present: self
        .setup
        .as_ref()
        .map(|s| !s.admin_username.trim().is_empty())
        .unwrap_or(false),
      scb_url_present: !self.scb.url.trim().is_empty(),
    };
    let resolved = secrets::resolve(req, reader)?;

    if let Some(url) = resolved.database_url {
      self.database.set_connection(url);
    }
    if let Some(conn) = resolved.smtp_connection {
      if let Some(email) = self.email.as_mut() {
        email.connection = conn;
      }
    }
    if let Some(pw) = resolved.redis_password {
      self.redis.password = Some(pw);
    }
    if let Some(pw) = resolved.admin_password {
      if let Some(setup) = self.setup.as_mut() {
        setup.admin_password = pw;
      }
    }
    if let Some(v) = resolved.scb_api_key {
      self.scb.api_key = v;
    }
    if let Some(v) = resolved.scb_api_secret {
      self.scb.api_secret = v;
    }
    if let Some(v) = resolved.scb_merchant_id {
      self.scb.merchant_id = v;
    }
    if let Some(v) = resolved.scb_terminal_id {
      self.scb.terminal_id = v;
    }
    Ok(())
  }

  pub fn get_database_url(&self) -> String {
    if let Ok(url) = env::var("app_108jobs_DATABASE_URL") {
      url
    } else {
      self.database.connection.clone()
    }
  }

  /// Returns either "http" or "https", depending on tls_enabled setting
  pub fn get_protocol_string(&self) -> &'static str {
    if self.tls_enabled {
      "https"
    } else {
      "http"
    }
  }

  /// Returns something like `http://localhost` or `https://app_108jobs.ml`,
  /// with the correct protocol and hostname.
  pub fn get_protocol_and_hostname(&self) -> String {
    format!("{}://{}", self.get_protocol_string(), self.hostname)
  }

  /// When running the federation test setup in `api_tests/` or `docker/federation`, the `hostname`
  /// variable will be like `app_108jobs-alpha:8541`. This method removes the port and returns
  /// `app_108jobs-alpha` instead. It has no effect in production.
  pub fn get_hostname_without_port(&self) -> Result<String, anyhow::Error> {
    Ok(
      (*self
        .hostname
        .split(':')
        .collect::<Vec<&str>>()
        .first()
        .context(location_info!())?)
      .to_string(),
    )
  }

  pub fn webfinger_regex(&self) -> Regex {
    WEBFINGER_REGEX.clone()
  }

  pub fn pictrs(&self) -> FastJobResult<PictrsConfig> {
    self
      .pictrs
      .clone()
      .ok_or_else(|| anyhow!("images_disabled").into())
  }
  pub fn get_phoenix_url(&self) -> &Option<Url> {
    &self.phoenix_url
  }

  pub fn get_redis_connection(&self) -> Result<String, anyhow::Error> {
    let conn_str = match &self.redis.password {
      Some(pwd) if !pwd.is_empty() => {
        format!("redis://:{}@{}:{}/", pwd, self.redis.host, self.redis.port)
      }
      _ => format!("redis://{}:{}/", self.redis.host, self.redis.port),
    };
    Ok(conn_str)
  }
}
#[allow(clippy::expect_used)]
/// Necessary to avoid URL expect failures
fn pictrs_placeholder_url() -> Url {
  Url::parse("http://localhost:8080").expect("parse pictrs url")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::settings::structs::{EmailConfig, SCBConfig, SetupConfig};
  use std::collections::HashMap;

  /// Default config file (after stripping secrets) must load without any env
  /// vars set — empty optional sections mean "feature disabled".
  ///
  /// We point `app_108jobs_CONFIG_LOCATION` at the workspace's `config/config.hjson`
  /// explicitly so the test passes whether cargo runs from the crate dir
  /// (`crates/utils`) or the workspace root.
  #[test]
  fn test_load_config() -> FastJobResult<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_config = format!("{manifest_dir}/../../config/config.hjson");
    // SAFETY: set_var is safe in single-threaded test setup. We only read it
    // via env::var inside the same test process.
    // Note: process-global env mutation can race with other tests, but no other
    // test in this crate touches this key.
    std::env::set_var("app_108jobs_CONFIG_LOCATION", &workspace_config);
    Settings::init()?;
    Ok(())
  }

  fn reader_from<'a>(
    map: &'a HashMap<&'a str, &'a str>,
  ) -> impl Fn(&str) -> Option<String> + Copy + 'a {
    move |k: &str| map.get(k).map(|s| s.to_string())
  }

  #[test]
  fn apply_env_secrets_email_active_requires_smtp_env() {
    let mut s = Settings::default();
    s.email = Some(EmailConfig {
      connection: String::new(),
      smtp_from_address: "noreply@example.com".to_string(),
    });
    s.database.set_connection("postgres://u@h/d".to_string());
    let map: HashMap<&str, &str> = HashMap::new();
    let err = s.apply_env_secrets(reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("SMTP_CONNECTION_URL"));
  }

  #[test]
  fn apply_env_secrets_scb_active_requires_all_scb_envs() {
    let mut s = Settings::default();
    s.scb = SCBConfig {
      url: "https://scb.example.com".to_string(),
      ..Default::default()
    };
    s.database.set_connection("postgres://u@h/d".to_string());
    let map: HashMap<&str, &str> = HashMap::new();
    let err = s.apply_env_secrets(reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("SCB_API_KEY"));
  }

  #[test]
  fn apply_env_secrets_setup_active_requires_admin_pw() {
    let mut s = Settings::default();
    s.setup = Some(SetupConfig {
      admin_username: "admin".to_string(),
      ..Default::default()
    });
    s.database.set_connection("postgres://u@h/d".to_string());
    let map: HashMap<&str, &str> = HashMap::new();
    let err = s.apply_env_secrets(reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("ADMIN_PASSWORD"));
  }

  #[test]
  fn apply_env_secrets_inactive_sections_skip_envs() {
    let mut s = Settings::default();
    s.database.set_connection("postgres://u@h/d".to_string());
    // No email/setup/scb activation triggers => no env vars required.
    let map: HashMap<&str, &str> = HashMap::new();
    s.apply_env_secrets(reader_from(&map)).unwrap();
  }

  #[test]
  fn apply_env_secrets_db_password_in_file_rejected() {
    let mut s = Settings::default();
    s.database
      .set_connection("postgres://user:hunter2@db/app".to_string());
    let map: HashMap<&str, &str> = HashMap::new();
    let err = s.apply_env_secrets(reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("appears to contain a password"));
  }
}
