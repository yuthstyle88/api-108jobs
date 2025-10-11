use crate::error::{FastJobErrorType, FastJobResult};
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use anyhow::Error;
use base64::engine::general_purpose;
use base64::Engine;
use p256::elliptic_curve;
use p256::elliptic_curve::sec1::FromEncodedPoint;
use p256::pkcs8::FromPrivateKey;
use p256::{ecdh::EphemeralSecret, EncodedPoint};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use spki::der::{Decodable, Encodable};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display, Formatter};
use tendril::fmt::Slice;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
  Secret,
  Private,
  Public,
}

#[derive(Serialize, Deserialize)]
pub struct RawKeyData(DataBuffer);

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct KeyData {
  pub(crate) r#type: KeyType,
  pub(crate) data: DataBuffer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeriveKeyArg {
  pub(crate) key: KeyData,
  pub(crate) public_key: Option<KeyData>,
}

pub struct ECParametersSpki {
  pub named_curve_alg: spki::der::asn1::ObjectIdentifier,
}

impl<'a> TryFrom<spki::der::asn1::Any<'a>> for ECParametersSpki {
  type Error = spki::der::Error;

  fn try_from(any: spki::der::asn1::Any<'a>) -> spki::der::Result<ECParametersSpki> {
    let x = any.oid()?;
    Ok(Self { named_curve_alg: x })
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataBuffer {
  pub buf: Buffer,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Buffer(Vec<u8>);

impl From<Vec<u8>> for Buffer {
  fn from(vec: Vec<u8>) -> Self {
    Self { 0: vec }
  }
}

impl DataBuffer {
  pub fn from_vec(buf: &[u8]) -> Self {
    Self {
      buf: Buffer(buf.to_vec()),
    }
  }
  pub fn from_str(astr: &str) -> Self {
    Self {
      buf: Buffer(astr.as_bytes().to_vec()),
    }
  }
}

/// Build Aes256Gcm from a hex key. Returns error if length != 32 bytes.
pub fn aes256_from_hex_key(hex_key: &str) -> FastJobResult<Aes256Gcm> {
  let key_bytes = hex::decode(hex_key).map_err(|_| FastJobErrorType::DecodeError)?;
  if key_bytes.len() != 32 {
    return Err(FastJobErrorType::InvalidLength.into());
  }
  let key: Key<Aes256Gcm> = GenericArray::from_slice(&key_bytes).clone();
  Ok(Aes256Gcm::new(&key))
}

/// Encrypt plaintext bytes into base64(nonce12 || ciphertext)
pub fn aes_gcm_encrypt_b64(aes: &Aes256Gcm, plaintext: &[u8]) -> FastJobResult<String> {
  let mut nonce_bytes = [0u8; 12];
  OsRng.fill_bytes(&mut nonce_bytes);
  let nonce = Nonce::from_slice(&nonce_bytes);
  let ciphertext = aes
      .encrypt(nonce, plaintext)
      .map_err(|_| FastJobErrorType::EncryptingError)?;
  let mut combined = Vec::with_capacity(12 + ciphertext.len());
  combined.extend_from_slice(&nonce_bytes);
  combined.extend_from_slice(&ciphertext);
  Ok(general_purpose::STANDARD.encode(&combined))
}

/// Decrypt base64(nonce12 || ciphertext) -> plaintext bytes
pub fn aes_gcm_decrypt_b64(aes: &Aes256Gcm, b64: &str) -> FastJobResult<Vec<u8>> {
  let combined = general_purpose::STANDARD
      .decode(b64)
      .map_err(|_| FastJobErrorType::DecodeError)?;
  if combined.len() < 12 {
    return Err(FastJobErrorType::CiphertextTooShort.into());
  }
  let (nonce_bytes, ct) = combined.split_at(12);
  let nonce = Nonce::from_slice(nonce_bytes);
  let pt = aes
      .decrypt(nonce, ct)
      .map_err(|_| FastJobErrorType::DecryptingError)?;
  Ok(pt)
}

pub fn xchange_encrypt_data_gcm(data: &str, hex_secret_key: &str) -> FastJobResult<String> {
  let aes = aes256_from_hex_key(hex_secret_key)?;
  let out = aes_gcm_encrypt_b64(&aes, data.as_bytes())?;
  Ok(out)
}

pub fn xchange_decrypt_data_gcm(encrypted_data: &str, hex_secret_key: &str) -> FastJobResult<String> {
  let aes = aes256_from_hex_key(hex_secret_key)?;
  let plaintext_bytes = aes_gcm_decrypt_b64(&aes, encrypted_data)?;
  let plaintext = String::from_utf8(plaintext_bytes)
      .map_err(|_| FastJobErrorType::DecodeError)?;
  Ok(plaintext)
}

pub fn import_public_key(data: DataBuffer) -> FastJobResult<Vec<u8>> {
  let pk_info = spki::SubjectPublicKeyInfo::from_der(&data.buf.0)
      .map_err(|_| FastJobErrorType::DecodeError)?;
  let alg = pk_info.algorithm.oid;
  if alg != elliptic_curve::ALGORITHM_OID {
    return Err(FastJobErrorType::InvalidAlgorithm.into());
  }
  let pk = pk_info.subject_public_key;
  let encoded_key = pk.to_vec();
  Ok(encoded_key)
}

pub fn export_public_key(data_buf: DataBuffer) -> FastJobResult<Vec<u8>> {
  let point = data_buf.buf;
  let subject_public_key = point.0.as_bytes();
  let alg_id = <p256::NistP256 as elliptic_curve::AlgorithmParameters>::algorithm_identifier();

  let key_info = spki::SubjectPublicKeyInfo {
    algorithm: alg_id,
    subject_public_key: &subject_public_key,
  };
  let spki_der = key_info
      .to_vec()
      .map_err(|_| FastJobErrorType::EncodeError)?;
  Ok(spki_der.into())
}

#[cfg(feature = "full")]
pub fn generate_key() -> FastJobResult<(EphemeralSecret, Vec<u8>)> {
  let secret = EphemeralSecret::random(&mut OsRng);
  let pk_bytes = EncodedPoint::from(secret.public_key());
  Ok((secret, pk_bytes.as_ref().to_vec()))
}

#[cfg(feature = "full")]
pub fn derive_key(args: DeriveKeyArg) -> FastJobResult<Vec<u8>> {
  let public_key = args
      .public_key
      .ok_or_else(|| FastJobErrorType::InvalidArgument)?;
  let secret_key = p256::SecretKey::from_pkcs8_der(args.key.data.buf.0.as_bytes())
      .map_err(|_| FastJobErrorType::DecodeError)?;
  let public_key = match public_key.r#type {
    KeyType::Private => p256::SecretKey::from_pkcs8_der(&public_key.data.buf.0)
        .map_err(|_| FastJobErrorType::DecodeError)?
        .public_key(),
    KeyType::Public => {
      let point = EncodedPoint::from_bytes(public_key.data.buf.0)
          .map_err(|_| FastJobErrorType::DecodeError)?;
      p256::PublicKey::from_encoded_point(&point)
          .ok_or_else(|| FastJobErrorType::DecodeError)?
    }
    _ => return Err(FastJobErrorType::InvalidArgument.into()),
  };
  let shared_secret = p256::elliptic_curve::ecdh::diffie_hellman(
    secret_key.to_secret_scalar(),
    public_key.as_affine(),
  );
  Ok(shared_secret.as_bytes().to_vec().into())
}

#[cfg(feature = "full")]
pub fn derive_secret_key(private_key: KeyData, public_key: Option<KeyData>) -> FastJobResult<String> {
  let args = DeriveKeyArg {
    key: private_key,
    public_key,
  };
  let secure_key = derive_key(args)?;
  let res = hex::encode(&secure_key);
  Ok(res)
}

#[cfg(feature = "full")]
pub fn xchange_encrypt_data(data: &str, hex_secret_key: &str, _session: &str) -> FastJobResult<String> {
  xchange_encrypt_data_gcm(data, hex_secret_key)
}

#[cfg(feature = "full")]
pub fn xchange_decrypt_data(
  encrypted_data: &str,
  hex_secret_key: &str,
  _session: &str,
) -> FastJobResult<String> {
  xchange_decrypt_data_gcm(encrypted_data, hex_secret_key)
}

pub type AnyError = anyhow::Error;

#[derive(Debug)]
struct FastJobError {
  message: Cow<'static, str>,
}

impl Display for FastJobError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(&self.message)
  }
}

impl std::error::Error for FastJobError {}

pub fn custom_error(_class: &'static str, message: impl Into<Cow<'static, str>>) -> Error {
  FastJobError {
    message: message.into(),
  }
      .into()
}

pub fn data_error(message: impl Into<Cow<'static, str>>) -> Error {
  custom_error("Error", message)
}

pub fn type_error(message: impl Into<Cow<'static, str>>) -> Error {
  custom_error("Error", message)
}