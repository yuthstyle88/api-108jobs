//! Environment-only secret loading and validation.
//!
//! All sensitive values (database password, SMTP credentials, admin password,
//! SCB API keys, Redis password) are loaded from environment variables and
//! validated at startup. The on-disk config file MUST NOT contain plaintext
//! secrets. Missing required secrets fail startup with a clear error.
//!
//! Env vars (lowercase prefix matches the rest of the project):
//!
//! | Variable                          | Required when                                 |
//! |-----------------------------------|-----------------------------------------------|
//! | `app_108jobs_DATABASE_URL`        | always, if `database.connection` is unset     |
//! | `app_108jobs_SMTP_CONNECTION_URL` | `email` section is configured                 |
//! | `app_108jobs_REDIS_PASSWORD`      | only if Redis instance requires auth          |
//! | `app_108jobs_ADMIN_PASSWORD`      | `setup` section is configured (first boot)    |
//! | `app_108jobs_SCB_API_KEY`         | `scb.url` is set                              |
//! | `app_108jobs_SCB_API_SECRET`      | `scb.url` is set                              |
//! | `app_108jobs_SCB_MERCHANT_ID`     | `scb.url` is set                              |
//! | `app_108jobs_SCB_TERMINAL_ID`     | `scb.url` is set                              |
//! | `app_108jobs_JWT_TTL_HOURS`       | optional, default 24                          |

use crate::error::{FastJobErrorType, FastJobResult};
use std::env;

/// Env-var key prefix. Lowercase to match the rest of the project.
pub const ENV_PREFIX: &str = "app_108jobs_";

/// Names of the env vars this module reads. Kept in one place to avoid drift.
pub mod env_keys {
  pub const DATABASE_URL: &str = "app_108jobs_DATABASE_URL";
  pub const SMTP_CONNECTION_URL: &str = "app_108jobs_SMTP_CONNECTION_URL";
  pub const REDIS_PASSWORD: &str = "app_108jobs_REDIS_PASSWORD";
  pub const ADMIN_PASSWORD: &str = "app_108jobs_ADMIN_PASSWORD";
  pub const SCB_API_KEY: &str = "app_108jobs_SCB_API_KEY";
  pub const SCB_API_SECRET: &str = "app_108jobs_SCB_API_SECRET";
  pub const SCB_MERCHANT_ID: &str = "app_108jobs_SCB_MERCHANT_ID";
  pub const SCB_TERMINAL_ID: &str = "app_108jobs_SCB_TERMINAL_ID";
  pub const JWT_TTL_HOURS: &str = "app_108jobs_JWT_TTL_HOURS";
  pub const ALLOW_WILDCARD_CORS: &str = "app_108jobs_ALLOW_WILDCARD_CORS";
}

/// Suspicious placeholder values we refuse in production. Catches the case
/// where someone copies the dev `config.hjson` into prod without overriding.
const REJECTED_PLACEHOLDERS: &[&str] = &[
  "changeme",
  "CHANGEME",
  "todo",
  "TODO",
  "<set-via-env>",
  "<unset>",
];

fn config_err(msg: impl Into<String>) -> FastJobErrorType {
  FastJobErrorType::Unknown(format!("config: {}", msg.into()))
}

/// Read an env var. Empty strings are treated as unset, on purpose: it is far
/// too easy to ship a Docker env file with `app_108jobs_SCB_API_KEY=` and we
/// would rather fail fast than authenticate with the literal empty string.
pub fn read_env(key: &str) -> Option<String> {
  read_env_via(key, |k| env::var(k).ok())
}

/// Variant that takes an explicit reader. Used by tests so we don't mutate
/// process-global env vars in parallel.
pub fn read_env_via<F>(key: &str, reader: F) -> Option<String>
where
  F: Fn(&str) -> Option<String>,
{
  match reader(key) {
    Some(v) if v.trim().is_empty() => None,
    Some(v) => Some(v),
    None => None,
  }
}

/// Reject a value if it matches a known placeholder. Returns Ok(value) if safe.
pub fn reject_placeholder(key: &str, value: String) -> FastJobResult<String> {
  if REJECTED_PLACEHOLDERS
    .iter()
    .any(|p| value.trim().eq_ignore_ascii_case(p))
  {
    Err(
      config_err(format!(
        "{} is set to a placeholder ({}). Refusing to start.",
        key,
        value.trim()
      ))
      .into(),
    )
  } else {
    Ok(value)
  }
}

/// Validate that a Postgres connection URL is at least syntactically plausible.
/// We do NOT contact the DB here — that happens later when the pool is built.
pub fn validate_postgres_url(url: &str) -> FastJobResult<()> {
  if !(url.starts_with("postgres://") || url.starts_with("postgresql://")) {
    return Err(
      config_err(format!(
        "database url must start with 'postgres://' or 'postgresql://' (got '{}')",
        first_chars(url, 32)
      ))
      .into(),
    );
  }
  Ok(())
}

/// Validate that the SMTP URL is syntactically plausible.
pub fn validate_smtp_url(url: &str) -> FastJobResult<()> {
  if !(url.starts_with("smtp://") || url.starts_with("smtps://")) {
    return Err(
      config_err(format!(
        "smtp connection url must start with 'smtp://' or 'smtps://' (got '{}')",
        first_chars(url, 32)
      ))
      .into(),
    );
  }
  Ok(())
}

fn first_chars(s: &str, n: usize) -> String {
  s.chars().take(n).collect()
}

/// Result of resolving env-injected secrets against a parsed [`Settings`].
/// Caller applies these values back into the [`Settings`] struct.
#[derive(Debug, Default)]
pub struct ResolvedSecrets {
  pub database_url: Option<String>,
  pub smtp_connection: Option<String>,
  pub redis_password: Option<String>,
  pub admin_password: Option<String>,
  pub scb_api_key: Option<String>,
  pub scb_api_secret: Option<String>,
  pub scb_merchant_id: Option<String>,
  pub scb_terminal_id: Option<String>,
}

/// Inputs from the parsed (non-secret) config that tell us which sections
/// are in use and therefore which env vars are required.
#[derive(Debug, Clone, Copy)]
pub struct SecretRequirements<'a> {
  pub database_connection_in_file: &'a str,
  pub email_section_present: bool,
  pub setup_section_present: bool,
  pub scb_url_present: bool,
}

/// Read all relevant env vars and decide whether the configuration is safe to
/// run. Pure: no panics, no process exits. Returns `Err` with a human-readable
/// message that gets surfaced at startup.
pub fn resolve<F>(req: SecretRequirements<'_>, reader: F) -> FastJobResult<ResolvedSecrets>
where
  F: Fn(&str) -> Option<String> + Copy,
{
  let mut out = ResolvedSecrets::default();

  // -- Database --
  // The file-level connection may be a dev placeholder. Env always wins.
  let file_conn = req.database_connection_in_file.trim();
  if let Some(url) = read_env_via(env_keys::DATABASE_URL, reader) {
    let url = reject_placeholder(env_keys::DATABASE_URL, url)?;
    validate_postgres_url(&url)?;
    out.database_url = Some(url);
  } else if !file_conn.is_empty() {
    // Allow the file value, but it MUST NOT contain a password.
    if file_conn_contains_password(file_conn) {
      return Err(
        config_err(format!(
          "{} not set and database.connection in config file appears to contain a password. \
         Move the password to env var {}.",
          env_keys::DATABASE_URL,
          env_keys::DATABASE_URL
        ))
        .into(),
      );
    }
    validate_postgres_url(file_conn)?;
  } else {
    return Err(
      config_err(format!(
        "no database connection: set {} or database.connection in config",
        env_keys::DATABASE_URL
      ))
      .into(),
    );
  }

  // -- SMTP --
  if req.email_section_present {
    let conn = read_env_via(env_keys::SMTP_CONNECTION_URL, reader).ok_or_else(|| {
      config_err(format!(
        "email section is configured but {} is not set",
        env_keys::SMTP_CONNECTION_URL
      ))
    })?;
    let conn = reject_placeholder(env_keys::SMTP_CONNECTION_URL, conn)?;
    validate_smtp_url(&conn)?;
    out.smtp_connection = Some(conn);
  }

  // -- Redis (optional) --
  if let Some(pw) = read_env_via(env_keys::REDIS_PASSWORD, reader) {
    let pw = reject_placeholder(env_keys::REDIS_PASSWORD, pw)?;
    out.redis_password = Some(pw);
  }

  // -- Admin (only required on first boot when setup is configured) --
  if req.setup_section_present {
    let pw = read_env_via(env_keys::ADMIN_PASSWORD, reader).ok_or_else(|| {
      config_err(format!(
        "setup section is configured but {} is not set",
        env_keys::ADMIN_PASSWORD
      ))
    })?;
    let pw = reject_placeholder(env_keys::ADMIN_PASSWORD, pw)?;
    if pw.len() < 10 || pw.len() > 60 {
      return Err(
        config_err(format!(
          "{} must be 10..=60 chars (got {})",
          env_keys::ADMIN_PASSWORD,
          pw.len()
        ))
        .into(),
      );
    }
    out.admin_password = Some(pw);
  }

  // -- SCB (all-or-nothing if URL is configured) --
  if req.scb_url_present {
    out.scb_api_key = Some(require_env(env_keys::SCB_API_KEY, reader)?);
    out.scb_api_secret = Some(require_env(env_keys::SCB_API_SECRET, reader)?);
    out.scb_merchant_id = Some(require_env(env_keys::SCB_MERCHANT_ID, reader)?);
    out.scb_terminal_id = Some(require_env(env_keys::SCB_TERMINAL_ID, reader)?);
  }

  Ok(out)
}

fn require_env<F>(key: &str, reader: F) -> FastJobResult<String>
where
  F: Fn(&str) -> Option<String>,
{
  let v = read_env_via(key, reader)
    .ok_or_else(|| config_err(format!("required env var {} is not set", key)))?;
  reject_placeholder(key, v)
}

/// Naive heuristic to detect `postgres://user:password@host/db` style URLs.
/// We do not parse fully — `url::Url` would reject some valid pg URLs.
fn file_conn_contains_password(conn: &str) -> bool {
  // strip scheme
  let after_scheme = conn.split_once("://").map(|(_, rest)| rest).unwrap_or(conn);
  // userinfo lives before the first '@', and contains a ':' only if a password is present
  if let Some(userinfo) = after_scheme.split('@').next() {
    if userinfo == after_scheme {
      // no '@' at all -> no userinfo
      return false;
    }
    return userinfo.contains(':');
  }
  false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  fn reader_from<'a>(
    map: &'a HashMap<&'a str, &'a str>,
  ) -> impl Fn(&str) -> Option<String> + Copy + 'a {
    move |k: &str| map.get(k).map(|s| s.to_string())
  }

  fn baseline_req() -> SecretRequirements<'static> {
    SecretRequirements {
      database_connection_in_file: "",
      email_section_present: false,
      setup_section_present: false,
      scb_url_present: false,
    }
  }

  #[test]
  fn missing_database_url_fails() {
    let map: HashMap<&str, &str> = HashMap::new();
    let err = resolve(baseline_req(), reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("no database connection"));
  }

  #[test]
  fn malformed_database_url_fails() {
    let map: HashMap<&str, &str> = HashMap::from([(env_keys::DATABASE_URL, "mysql://x/y")]);
    let err = resolve(baseline_req(), reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("must start with 'postgres"));
  }

  #[test]
  fn empty_database_url_treated_as_unset() {
    let map: HashMap<&str, &str> = HashMap::from([(env_keys::DATABASE_URL, "   ")]);
    let err = resolve(baseline_req(), reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("no database connection"));
  }

  #[test]
  fn file_connection_with_password_rejected() {
    let req = SecretRequirements {
      database_connection_in_file: "postgres://user:secret@db/app",
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::new();
    let err = resolve(req, reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("appears to contain a password"));
  }

  #[test]
  fn file_connection_without_password_ok() {
    let req = SecretRequirements {
      database_connection_in_file: "postgres://user@db/app",
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::new();
    let r = resolve(req, reader_from(&map)).unwrap();
    assert!(r.database_url.is_none()); // env did not override
  }

  #[test]
  fn placeholder_rejected() {
    let map: HashMap<&str, &str> = HashMap::from([(env_keys::DATABASE_URL, "changeme")]);
    let err = resolve(baseline_req(), reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("placeholder"));
  }

  #[test]
  fn email_section_requires_smtp_env() {
    let req = SecretRequirements {
      email_section_present: true,
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::from([(env_keys::DATABASE_URL, "postgres://x/y")]);
    let err = resolve(req, reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("SMTP_CONNECTION_URL"));
  }

  #[test]
  fn smtp_url_malformed_rejected() {
    let req = SecretRequirements {
      email_section_present: true,
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::from([
      (env_keys::DATABASE_URL, "postgres://x/y"),
      (env_keys::SMTP_CONNECTION_URL, "http://not-smtp"),
    ]);
    let err = resolve(req, reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("smtp connection url must start"));
  }

  #[test]
  fn admin_password_required_when_setup_present() {
    let req = SecretRequirements {
      setup_section_present: true,
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::from([(env_keys::DATABASE_URL, "postgres://x/y")]);
    let err = resolve(req, reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("ADMIN_PASSWORD"));
  }

  #[test]
  fn admin_password_too_short_rejected() {
    let req = SecretRequirements {
      setup_section_present: true,
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::from([
      (env_keys::DATABASE_URL, "postgres://x/y"),
      (env_keys::ADMIN_PASSWORD, "short"),
    ]);
    let err = resolve(req, reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("10..=60"));
  }

  #[test]
  fn admin_password_accepted() {
    let req = SecretRequirements {
      setup_section_present: true,
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::from([
      (env_keys::DATABASE_URL, "postgres://x/y"),
      (env_keys::ADMIN_PASSWORD, "1234567890ab"),
    ]);
    let r = resolve(req, reader_from(&map)).unwrap();
    assert_eq!(r.admin_password.as_deref(), Some("1234567890ab"));
  }

  #[test]
  fn scb_all_or_nothing_partial_fails() {
    let req = SecretRequirements {
      scb_url_present: true,
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::from([
      (env_keys::DATABASE_URL, "postgres://x/y"),
      (env_keys::SCB_API_KEY, "k"),
      // SCB_API_SECRET, MERCHANT_ID, TERMINAL_ID missing
    ]);
    let err = resolve(req, reader_from(&map)).unwrap_err();
    assert!(format!("{err:?}").contains("SCB_API_SECRET"));
  }

  #[test]
  fn scb_all_present_ok() {
    let req = SecretRequirements {
      scb_url_present: true,
      ..baseline_req()
    };
    let map: HashMap<&str, &str> = HashMap::from([
      (env_keys::DATABASE_URL, "postgres://x/y"),
      (env_keys::SCB_API_KEY, "k"),
      (env_keys::SCB_API_SECRET, "s"),
      (env_keys::SCB_MERCHANT_ID, "m"),
      (env_keys::SCB_TERMINAL_ID, "t"),
    ]);
    let r = resolve(req, reader_from(&map)).unwrap();
    assert_eq!(r.scb_api_key.as_deref(), Some("k"));
    assert_eq!(r.scb_api_secret.as_deref(), Some("s"));
    assert_eq!(r.scb_merchant_id.as_deref(), Some("m"));
    assert_eq!(r.scb_terminal_id.as_deref(), Some("t"));
  }

  #[test]
  fn file_conn_password_detector() {
    assert!(file_conn_contains_password("postgres://u:p@h/d"));
    assert!(!file_conn_contains_password("postgres://u@h/d"));
    assert!(!file_conn_contains_password("postgres://h/d"));
    assert!(!file_conn_contains_password("postgres://h:5432/d"));
  }
}
