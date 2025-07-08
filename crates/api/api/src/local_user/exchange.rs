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

pub async fn exchange_keys(
  data: Json<ExchangeKey>,
  _req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<ExchangeKeyResponse>> {
  let server_public_key: SensitiveString;
  let secret = context.crypto_secret();
  let token = data
    .token
    .as_ref()
    .ok_or(FastJobErrorType::IncorrectTotpToken)?;
  let local_user_id = Claims::validate(token, context.get_ref()).await;
  if let Ok((user_id, _session)) = local_user_id {
    server_public_key = hex::encode(context.public_key()).into();
    let client_public_key = &data.public_key;

    let decoded =
      hex::decode(client_public_key.to_owned()).map_err(|_| FastJobErrorType::DecodeError)?;
    let client_data_buf = DataBuffer::from_vec(&decoded);

    let temp = Crypto::import_public_key(client_data_buf)?;
    let public_key =
      PublicKey::from_sec1_bytes(temp.as_ref()).map_err(|_| FastJobErrorType::DecodeError)?;

    let secret_key = secret.diffie_hellman(&public_key);
    let client_hex_secret_key = hex::encode(secret_key.as_bytes().to_vec());
    let _ = LocalUser::update_public_key(&mut context.pool(), user_id, &client_hex_secret_key);
  } else {
    server_public_key = SensitiveString::from("token_error".to_string());
  }

  let res = ExchangeKeyResponse {
    public_key: server_public_key,
  };
  Ok(Json(res))
}

