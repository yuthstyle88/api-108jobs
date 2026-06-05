use actix_cors::Cors;
use app_108jobs_utils::settings::structs::Settings;
use std::env;
use std::path::{Path, PathBuf};

pub mod prometheus_metrics;
pub mod scheduled_tasks;
pub mod setup_local_site;

/// Build the CORS layer.
///
/// Wildcard origins are intentionally rejected in release builds unless the
/// operator opts in with `app_108jobs_ALLOW_WILDCARD_CORS=1`. In debug builds
/// we still permit `*` to keep local development frictionless.
///
/// Empty origin list in release mode falls back to *self-only*, which is
/// safer than the previous behavior of silently allowing any origin.
pub fn cors_config(settings: &Settings) -> Cors {
  let self_origin = settings.get_protocol_and_hostname();
  let cors_origin_setting = settings.cors_origin();
  let allow_wildcard_opt_in = matches!(
    env::var("app_108jobs_ALLOW_WILDCARD_CORS").as_deref(),
    Ok("1") | Ok("true") | Ok("TRUE")
  );

  let mut cors = Cors::default()
    .allow_any_method()
    .allow_any_header()
    .expose_any_header()
    .max_age(3600);

  let contains_wildcard = cors_origin_setting.contains(&"*".to_string());

  if cfg!(debug_assertions) {
    // Local dev: permissive — matches the old behavior to keep onboarding easy.
    cors = cors.allow_any_origin();
  } else if contains_wildcard && allow_wildcard_opt_in {
    tracing::warn!(
      "CORS wildcard origin enabled by app_108jobs_ALLOW_WILDCARD_CORS env var. \
       Do not use this in production."
    );
    cors = cors.allow_any_origin();
  } else if contains_wildcard {
    tracing::error!(
      "cors_origin contains '*' but app_108jobs_ALLOW_WILDCARD_CORS is unset; \
       falling back to self-origin only ({}). Set explicit origins in config \
       to allow cross-origin requests in production.",
      self_origin
    );
    cors = cors.allowed_origin(&self_origin);
  } else if cors_origin_setting.is_empty() {
    cors = cors.allowed_origin(&self_origin);
  } else {
    cors = cors.allowed_origin(&self_origin);
    for c in cors_origin_setting {
      cors = cors.allowed_origin(&c);
    }
  }
  cors
}

pub fn unique_target_filename(orig: &str) -> String {
  let ts = chrono::Utc::now().timestamp_millis();
  if let Some((stem, ext)) = orig.rsplit_once('.') {
    format!("{}_{}.{}", stem, ts, ext)
  } else {
    format!("{}_{}", orig, ts)
  }
}

pub fn user_files_dir(user_id: i32) -> PathBuf {
  PathBuf::from("uploads")
    .join("files")
    .join(user_id.to_string())
}

pub fn sanitize_filename(name: &str) -> String {
  let name = name.trim();
  // Strip any path components and keep only a-zA-Z0-9 . _ -
  let base = Path::new(name)
    .file_name()
    .unwrap_or_default()
    .to_string_lossy();
  base
    .chars()
    .map(|c| match c {
      'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => c,
      _ => '-',
    })
    .collect()
}
