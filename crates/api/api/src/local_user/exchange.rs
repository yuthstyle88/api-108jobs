use actix_web::{
  web::{Data, Json},
  HttpRequest,
};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::sensitive::SensitiveString;
use app_108jobs_db_schema::source::person::Person;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::{ExchangeKey, ExchangeKeyResponse};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use p256::{PublicKey, SecretKey};

use app_108jobs_utils::crypto::{
  normalize_pubkey_to_uncompressed_hex,
  public_key_to_hex,
  derive_aes256_from_ecdh,
  export_private_pkcs8_der,
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use rand::rngs::OsRng;
use hex;

// Per-process ephemeral ECDH secrets, keyed by local user id
static SERVER_EPHEMERAL: OnceLock<Mutex<HashMap<i64, SecretKey>>> = OnceLock::new();

// Per-process derived AES-256 session keys, keyed by local user id
static SERVER_SESSION_KEYS: OnceLock<Mutex<HashMap<i64, [u8; 32]>>> = OnceLock::new();

pub async fn exchange_key(
  data: Json<ExchangeKey>,
  _req: HttpRequest,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView
) -> FastJobResult<Json<ExchangeKeyResponse>> {
  // Read current person
  // Accept both hex or base64, compressed or uncompressed SEC1, normalize to uncompressed-hex before storing
  let raw_in = data.public_key.trim();
  // Normalize any (hex/base64, compressed/uncompressed) input to uncompressed-hex for storage
  let client_hex = normalize_pubkey_to_uncompressed_hex(raw_in)
    .map_err(|_| FastJobErrorType::DecodeError)?;


  // Get per-process ephemeral server secret for this user, or create one
  let map = SERVER_EPHEMERAL.get_or_init(|| Mutex::new(HashMap::new()));
  let user_id_i32: i32 = local_user_view.person.id.0;
  let server_secret = {
    let mut guard = map.lock().unwrap();
    guard.entry(user_id_i32 as i64).or_insert_with(|| SecretKey::random(&mut OsRng)).clone()
  };
  // Derive the server's public key (uncompressed) first
  let server_public: PublicKey = server_secret.public_key();
  let server_public_hex = public_key_to_hex(&server_public);

  // Derive and store an AES-256 session key for this user (per-process)
  // 1) decode client pubkey (uncompressed hex -> raw bytes)
  let client_pub_raw = hex::decode(&client_hex).map_err(|_| FastJobErrorType::DecodeError)?;

  // 2) export server secret to DER and derive shared AES key
  let server_sk_der = export_private_pkcs8_der(&server_secret);
  let aes_key = derive_aes256_from_ecdh(&server_sk_der, &client_pub_raw)
    .map_err(|_| FastJobErrorType::EncryptingError)?;

  // Persist the derived session shared key (hex) for this user
  let server_share_key = hex::encode(&aes_key);
  let _ = Person::update_share_key(
    &mut context.pool(),
    local_user_view.person.id,
    &server_share_key,
  ).await;

  // 3) store the derived key in memory keyed by user id (consider Redis for multi-instance)
  let key_map = SERVER_SESSION_KEYS.get_or_init(|| Mutex::new(HashMap::new()));
  {
    let mut g = key_map.lock().unwrap();
    g.insert(user_id_i32 as i64, aes_key);
  }

  Ok(Json(ExchangeKeyResponse {
    public_key: SensitiveString::from(server_public_hex),
  }))
}
