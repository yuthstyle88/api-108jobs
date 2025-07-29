use crate::context::FastJobContext;
use actix_web::{http::header::USER_AGENT, HttpRequest};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lemmy_db_schema::{
  newtypes::LocalUserId,
  sensitive::SensitiveString,
  source::login_token::{LoginToken, LoginTokenCreateForm},
};
use lemmy_db_schema_file::enums::Role;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Claims {
  /// local_user_id, standard claim by RFC 7519.
  pub sub: String,
  pub iss: String,
  /// Time when this token was issued as UNIX-timestamp in seconds
  pub iat: i64,
  pub exp: i64,
  pub session: String,
  pub role: Role,
  pub email: Option<SensitiveString>,
  pub lang: String,
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
    role: Role,
    lang: String,
    req: HttpRequest,
    context: &FastJobContext,
  ) -> FastJobResult<SensitiveString> {
    let hostname = context.settings().hostname.clone();
    let my_claims = Claims {
      sub: user_id.0.to_string(),
      iss: hostname,
      iat: Utc::now().timestamp(),
      exp: (Utc::now() + Duration::hours(12)).timestamp(),
      session: generate_session(),
      role,
      email,
      lang,
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