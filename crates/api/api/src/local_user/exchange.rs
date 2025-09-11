use actix_web::{
  web::{Data, Json, Path},
  HttpRequest,
};
use lemmy_api_utils::claims::Claims;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::read_auth_token;
use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_db_schema::source::person::Person;
use lemmy_db_schema::newtypes::PersonId;
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{ExchangeKey, ExchangeKeyResponse, UserKeysResponse};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use p256::PublicKey;

pub async fn exchange_key(
  data: Json<ExchangeKey>,
  req: HttpRequest,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView
) -> FastJobResult<Json<ExchangeKeyResponse>> {
  // Validate token
  let jwt = read_auth_token(&req)?;

  let stored_key: String;
  if let Ok((_user_id, _session)) = Claims::validate(jwt.as_ref().map(|s| s.as_str()).unwrap_or(""), context.get_ref()).await {
    // Read current person and store client's public key only if not already set
    let person = Person::read(&mut context.pool(), local_user_view.person.id).await?;
    let is_placeholder = person.public_key.is_empty()
      || person.public_key == "public_key"
      || person.public_key == "pubkey";
    if is_placeholder {
      // Minimal validation: hex decode, SEC1 uncompressed length and prefix, parse via p256
      let hex_str = data.public_key.trim();
      let decoded = hex::decode(hex_str).map_err(|_| FastJobErrorType::DecodeError)?;
      if decoded.len() != 65 || decoded[0] != 0x04 {
        return Err(FastJobErrorType::DecodeError.into());
      }
      // Validate using p256 SEC1 parser
      let _ = PublicKey::from_sec1_bytes(&decoded).map_err(|_| FastJobErrorType::DecodeError)?;
      // Store client's identity public key as-is; don't overwrite if already exists
      let _ = Person::update_public_key(&mut context.pool(), local_user_view.person.id, hex_str).await;
      stored_key = hex_str.to_string();
    } else {
      stored_key = person.public_key;
    }
  } else {
    return Err(FastJobErrorType::NotLoggedIn.into());
  }

  Ok(Json(ExchangeKeyResponse {
    public_key: SensitiveString::from(stored_key),
  }))
}

pub async fn get_user_keys(
  path: Path<i32>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<UserKeysResponse>> {
  let person_id = PersonId(path.into_inner());
  let person = Person::read(&mut context.pool(), person_id).await?;
  let mut keys = Vec::new();
  if !person.public_key.is_empty() {
    keys.push(person.public_key);
  }
  Ok(Json(UserKeysResponse { public_keys: keys }))
}

