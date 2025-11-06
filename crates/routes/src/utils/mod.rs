use actix_cors::Cors;
use chrono::{DateTime, NaiveDateTime, Utc};
use lemmy_utils::settings::structs::Settings;
use std::path::{Path, PathBuf};

pub mod prometheus_metrics;
pub mod scheduled_tasks;
pub mod setup_local_site;

pub fn cors_config(settings: &Settings) -> Cors {
  let self_origin = settings.get_protocol_and_hostname();
  let cors_origin_setting = settings.cors_origin();

  let mut cors = Cors::default()
    .allow_any_method()
    .allow_any_header()
    .expose_any_header()
    .max_age(3600);

  if cfg!(debug_assertions)
    || cors_origin_setting.is_empty()
    || cors_origin_setting.contains(&"*".to_string())
  {
    cors = cors.allow_any_origin();
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

pub fn parse_expiry_time(s: &str) -> DateTime<Utc> {
  NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
    .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
    .or_else(|_| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)))
    .unwrap_or_else(|_| Utc::now()) // fallback if parsing fails
}
