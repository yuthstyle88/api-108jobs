use crate::context::FastJobContext;
use actix_web::{http::header::USER_AGENT, HttpRequest};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  newtypes::LocalUserId,
  sensitive::SensitiveString,
  source::login_token::{LoginToken, LoginTokenCreateForm},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

/// JWT lifetime in hours. Read from `app_108jobs_JWT_TTL_HOURS` if set,
/// otherwise 24h. Clamped to a sane range [1, 720] (1 hour .. 30 days).
/// The previous default (1200 h = 50 days) was unacceptably long for a
/// platform handling payments.
const JWT_DEFAULT_TTL_HOURS: i64 = 24;
const JWT_MIN_TTL_HOURS: i64 = 1;
const JWT_MAX_TTL_HOURS: i64 = 720;

pub fn jwt_ttl_hours() -> i64 {
  jwt_ttl_hours_from(|k| env::var(k).ok())
}

pub fn jwt_ttl_hours_from<F>(reader: F) -> i64
where
  F: Fn(&str) -> Option<String>,
{
  let raw = match reader("app_108jobs_JWT_TTL_HOURS") {
    Some(v) => v,
    None => return JWT_DEFAULT_TTL_HOURS,
  };
  let parsed: i64 = match raw.trim().parse() {
    Ok(n) => n,
    Err(_) => {
      tracing::warn!(
        "app_108jobs_JWT_TTL_HOURS={:?} is not a valid integer; using default {}",
        raw,
        JWT_DEFAULT_TTL_HOURS
      );
      return JWT_DEFAULT_TTL_HOURS;
    }
  };
  parsed.clamp(JWT_MIN_TTL_HOURS, JWT_MAX_TTL_HOURS)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
  /// local_user_id, standard claim by RFC 7519.
  pub sub: String,
  pub iss: String,
  /// Time when this token was issued as UNIX-timestamp in seconds
  pub iat: i64,
  pub exp: i64,
  pub session: String,
  pub email: Option<SensitiveString>,
  pub lang: String,
  pub accepted_terms: bool,
  pub is_admin: bool,
}

impl Claims {
  pub async fn validate(
    jwt: &str,
    context: &FastJobContext,
  ) -> FastJobResult<(LocalUserId, String)> {
    let validation = Validation::default();
    let jwt_secret = &context.secret().jwt_secret;
    let key = DecodingKey::from_secret(jwt_secret.as_ref());
    let claims =
      decode::<Claims>(jwt, &key, &validation).with_fastjob_type(FastJobErrorType::NotLoggedIn)?;
    let session = claims.claims.session.clone();
    let user_id = LocalUserId(claims.claims.sub.parse()?);
    LoginToken::validate(&mut context.pool(), user_id, jwt).await?;
    Ok((user_id, session))
  }

  pub async fn generate(
    user_id: LocalUserId,
    email: Option<SensitiveString>,
    lang: String,
    accepted_terms: bool,
    is_admin: bool,
    req: HttpRequest,
    context: &FastJobContext,
  ) -> FastJobResult<SensitiveString> {
    let hostname = context.settings().hostname.clone();
    let ttl_hours = jwt_ttl_hours();
    let my_claims = Claims {
      sub: user_id.0.to_string(),
      iss: hostname,
      iat: Utc::now().timestamp(),
      exp: (Utc::now() + Duration::hours(ttl_hours)).timestamp(),
      session: generate_session(),
      email,
      lang,
      accepted_terms,
      is_admin,
    };

    let secret = &context.secret().jwt_secret;
    let key = EncodingKey::from_secret(secret.as_ref());
    let token: SensitiveString = encode(&Header::default(), &my_claims, &key)?.into();
    let ip = req
      .connection_info()
      .realip_remote_addr()
      .map(ToString::to_string);
    let user_agent = req
      .headers()
      .get(USER_AGENT)
      .and_then(|ua| ua.to_str().ok())
      .map(ToString::to_string);
    let form = LoginTokenCreateForm {
      token: token.clone(),
      user_id,
      ip,
      user_agent,
    };
    LoginToken::create(&mut context.pool(), form).await?;
    Ok(token)
  }
}
pub fn generate_session() -> String {
  Uuid::new_v4().as_simple().to_string()
}

#[cfg(test)]
mod ttl_tests {
  use super::*;
  use std::collections::HashMap;

  fn reader_from<'a>(map: &'a HashMap<&'a str, &'a str>) -> impl Fn(&str) -> Option<String> + 'a {
    move |k: &str| map.get(k).copied().map(ToString::to_string)
  }

  #[test]
  fn ttl_defaults_to_24_when_unset() {
    let map: HashMap<&str, &str> = HashMap::new();
    assert_eq!(jwt_ttl_hours_from(reader_from(&map)), 24);
  }

  #[test]
  fn ttl_uses_env_value() {
    let map = HashMap::from([("app_108jobs_JWT_TTL_HOURS", "48")]);
    assert_eq!(jwt_ttl_hours_from(reader_from(&map)), 48);
  }

  #[test]
  fn ttl_clamps_to_max_30_days() {
    let map = HashMap::from([("app_108jobs_JWT_TTL_HOURS", "5000")]);
    assert_eq!(jwt_ttl_hours_from(reader_from(&map)), 720);
  }

  #[test]
  fn ttl_clamps_to_min_one_hour() {
    let map = HashMap::from([("app_108jobs_JWT_TTL_HOURS", "0")]);
    assert_eq!(jwt_ttl_hours_from(reader_from(&map)), 1);
  }

  #[test]
  fn ttl_garbage_falls_back_to_default() {
    let map = HashMap::from([("app_108jobs_JWT_TTL_HOURS", "abc")]);
    assert_eq!(jwt_ttl_hours_from(reader_from(&map)), 24);
  }

  #[test]
  fn ttl_old_default_1200_no_longer_in_use() {
    // Sanity guard against regression to the audit-flagged value.
    let map: HashMap<&str, &str> = HashMap::new();
    assert!(jwt_ttl_hours_from(reader_from(&map)) < 1200);
  }
}
