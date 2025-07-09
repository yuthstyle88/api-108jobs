use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use lemmy_api_utils::claims::Claims;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_db_views_site::api::{ExchangeKey, ExchangeKeyResponse};
use lemmy_utils::crypto::{Crypto, DataBuffer};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use p256::PublicKey;
pub async fn exchange_key(
  data: Json<ExchangeKey>,
  _req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<ExchangeKeyResponse>> {
  // Validate token
  let token = data
   .token
   .as_ref()
   .ok_or(FastJobErrorType::IncorrectTotpToken)?;

  // Generate server keypair
  let (server_secret, server_public_raw) = Crypto::generate_key()?;
  let server_public_buffer = DataBuffer::from_vec(server_public_raw.as_bytes());
  let server_public_encoded = Crypto::export_public_key(server_public_buffer)?;
  let server_public_hex: SensitiveString = hex::encode(&server_public_encoded);

  // Validate user and process client key
  if let Ok((user_id, _session)) = Claims::validate(token, context.get_ref()).await {
    // Process client public key
    let client_public_raw = hex::decode(&data.public_key)
     .map_err(|_| FastJobErrorType::DecodeError)?;
    let client_public_buffer = DataBuffer::from_vec(&client_public_raw);
    let client_public_encoded = Crypto::import_public_key(client_public_buffer)?;
    let client_public_key = PublicKey::from_sec1_bytes(&client_public_encoded)
     .map_err(|_| FastJobErrorType::DecodeError)?;

    // Generate shared secret
    let shared_secret = server_secret.diffie_hellman(&client_public_key);
    let shared_secret_hex = hex::encode(shared_secret.as_bytes());

    // Update user's public key
    let _ = LocalUser::update_public_key(&mut context.pool(), user_id, &shared_secret_hex);
  }

  Ok(Json(ExchangeKeyResponse {
    public_key: server_public_hex,
  }))
}

