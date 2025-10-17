use aes_gcm::aead::rand_core::{OsRng, RngCore};
use aes_gcm::Nonce;
use aes_gcm::{aead::{generic_array::GenericArray, Aead, Payload}, Aes256Gcm, KeyInit};
use base64::engine::general_purpose;
use base64::Engine;
use hex;
use p256::ecdh::diffie_hellman;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};

/// ====== Errors ======
#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
  #[error("decode error")]
  Decode,
  #[error("invalid length")]
  InvalidLength,
  #[error("invalid key")]
  InvalidKey,
  #[error("ciphertext too short")]
  CiphertextTooShort,
  #[error("encrypt error")]
  Encrypt,
  #[error("decrypt error")]
  Decrypt,
}

/// ====== ECDH (P-256) helpers ======

/// Common sizes
pub const AES_KEY_LEN: usize = 32;
pub const GCM_NONCE_LEN: usize = 12;

/// A compact wire-friendly box: base64(iv) + base64(ciphertext|tag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcmBox {
  /// 12-byte GCM nonce, base64-encoded
  pub iv: String,
  /// ciphertext with tag appended, base64-encoded
  pub ct: String,
}

/// Generate a fresh P-256 keypair.
/// Returns (private_key, public_key_uncompressed_raw_65B)
pub fn ecdh_generate_keypair() -> (SecretKey, Vec<u8>) {
  let sk = SecretKey::random(&mut OsRng);
  let pk: PublicKey = sk.public_key();
  let ep = pk.to_encoded_point(false); // uncompressed (0x04 + X + Y) = 65 bytes
  (sk, ep.as_bytes().to_vec())
}

/// Export SecretKey to PKCS#8 DER bytes (for storage if needed).
pub fn export_private_pkcs8_der(sk: &SecretKey) -> Vec<u8> {
  use p256::pkcs8::EncodePrivateKey;
  sk.to_pkcs8_der().expect("pkcs8 encode").as_bytes().to_vec()
}

/// Import SecretKey from PKCS#8 DER bytes.
pub fn import_private_pkcs8_der(der: &[u8]) -> Result<SecretKey, CryptoError> {
  use p256::pkcs8::DecodePrivateKey;
  SecretKey::from_pkcs8_der(der).map_err(|_| CryptoError::Decode)
}

/// Import peer public key from either uncompressed (65B) or compressed (33B) SEC1 raw bytes.
pub fn import_peer_public_key_raw(bytes: &[u8]) -> Result<PublicKey, CryptoError> {
  // Accept compressed (33B) or uncompressed (65B) SEC1 encodings
  PublicKey::from_sec1_bytes(bytes).map_err(|_| CryptoError::InvalidKey)
}

/// Convert a public key to uncompressed SEC1 raw (65B).
pub fn public_key_to_uncompressed_raw(pk: &PublicKey) -> Vec<u8> {
  pk.to_encoded_point(false).as_bytes().to_vec()
}

/// Export a public key to base64 (uncompressed SEC1 raw 65B).
pub fn public_key_to_base64(pk: &PublicKey) -> String {
  let raw = pk.to_encoded_point(false).as_bytes().to_vec();
  general_purpose::STANDARD.encode(raw)
}

/// Export a public key to hex (uncompressed SEC1 raw 65B).
pub fn public_key_to_hex(pk: &PublicKey) -> String {
  let raw = pk.to_encoded_point(false).as_bytes().to_vec();
  hex::encode(raw)
}

/// Import peer public key from base64 (accepts compressed 33B or uncompressed 65B).
pub fn public_key_from_base64(b64: &str) -> Result<PublicKey, CryptoError> {
  let raw = general_purpose::STANDARD.decode(b64).map_err(|_| CryptoError::Decode)?;
  import_peer_public_key_raw(&raw)
}

/// Import peer public key from hex (accepts compressed 33B or uncompressed 65B).
pub fn public_key_from_hex_str(hex_str: &str) -> Result<PublicKey, CryptoError> {
  let raw = hex::decode(hex_str.trim()).map_err(|_| CryptoError::Decode)?;
  import_peer_public_key_raw(&raw)
}

/// Normalize any (hex/base64, compressed/uncompressed) public key string
/// into uncompressed-hex (130 chars) for storage/UI consistency.
pub fn normalize_pubkey_to_uncompressed_hex(s: &str) -> Result<String, CryptoError> {
  // try hex first then base64
  let raw = match hex::decode(s.trim()) {
    Ok(b) => b,
    Err(_) => general_purpose::STANDARD.decode(s.trim()).map_err(|_| CryptoError::Decode)?,
  };
  let pk = import_peer_public_key_raw(&raw)?;
  let uncompressed = pk.to_encoded_point(false);
  Ok(hex::encode(uncompressed.as_bytes()))
}

/// Derive a 32-byte AES key directly from ECDH shared secret.
/// `my_sk_der` = PKCS#8 DER of our private key,
/// `peer_pub_raw` = peer public key SEC1 (compressed 33B or uncompressed 65B).
pub fn derive_aes256_from_ecdh(my_sk_der: &[u8], peer_pub_raw: &[u8]) -> Result<[u8; 32], CryptoError> {
  let my_sk = import_private_pkcs8_der(my_sk_der)?;
  let peer_pk = import_peer_public_key_raw(peer_pub_raw)?;
  // ECDH shared secret (P-256 gives 32 bytes)
  let scalar = my_sk.to_nonzero_scalar();
  let shared = diffie_hellman(&scalar, peer_pk.as_affine());
  let bytes = shared.raw_secret_bytes();
  let mut out = [0u8; 32];
  out.copy_from_slice(bytes.as_slice());
  Ok(out)
}

/// ====== AES-GCM-256 helpers ======

/// Build Aes256Gcm from a 32-byte key (AES-256).
pub fn aes256_from_key_bytes(key: &[u8]) -> Result<Aes256Gcm, CryptoError> {
  if key.len() != 32 { return Err(CryptoError::InvalidLength); }
  let k = GenericArray::from_slice(key).clone();
  Ok(Aes256Gcm::new(&k))
}

/// Encrypt with optional AAD, returning a structured box {iv, ct} (both base64).
pub fn aes_gcm_encrypt_box_b64(key: &[u8], plaintext: &[u8], aad: Option<&[u8]>) -> Result<GcmBox, CryptoError> {
  let aes = aes256_from_key_bytes(key)?;
  let mut nonce_bytes = [0u8; GCM_NONCE_LEN];
  OsRng.fill_bytes(&mut nonce_bytes);
  let nonce = Nonce::from_slice(&nonce_bytes);
  let payload = match aad {
    Some(a) => Payload { msg: plaintext, aad: a },
    None => Payload { msg: plaintext, aad: &[] },
  };
  let ct = aes.encrypt(nonce, payload).map_err(|_| CryptoError::Encrypt)?;
  Ok(GcmBox {
    iv: general_purpose::STANDARD.encode(nonce_bytes),
    ct: general_purpose::STANDARD.encode(ct),
  })
}

/// Decrypt a structured box {iv, ct} (both base64), with optional AAD.
pub fn aes_gcm_decrypt_box_b64(key: &[u8], boxed: &GcmBox, aad: Option<&[u8]>) -> Result<Vec<u8>, CryptoError> {
  let aes = aes256_from_key_bytes(key)?;
  let nonce_bytes = general_purpose::STANDARD.decode(&boxed.iv).map_err(|_| CryptoError::Decode)?;
  if nonce_bytes.len() != GCM_NONCE_LEN { return Err(CryptoError::CiphertextTooShort); }
  let ct = general_purpose::STANDARD.decode(&boxed.ct).map_err(|_| CryptoError::Decode)?;
  let nonce = Nonce::from_slice(&nonce_bytes);
  let payload = match aad {
    Some(a) => Payload { msg: ct.as_ref(), aad: a },
    None => Payload { msg: ct.as_ref(), aad: &[] },
  };
  let pt = aes.decrypt(nonce, payload).map_err(|_| CryptoError::Decrypt)?;
  Ok(pt)
}

/// Encrypt plaintext bytes. Returns base64(nonce12 || ciphertext_with_tag).
pub fn aes_gcm_encrypt_b64(key: &[u8], plaintext: &[u8]) -> Result<String, CryptoError> {
  let aes = aes256_from_key_bytes(key)?;
  let mut nonce_bytes = [0u8; GCM_NONCE_LEN];
  OsRng.fill_bytes(&mut nonce_bytes);
  let nonce = Nonce::from_slice(&nonce_bytes);
  let ct = aes.encrypt(nonce, Payload { msg: plaintext, aad: &[] }).map_err(|_| CryptoError::Encrypt)?;
  let mut out = Vec::with_capacity(GCM_NONCE_LEN + ct.len());
  out.extend_from_slice(&nonce_bytes);
  out.extend_from_slice(&ct);
  Ok(general_purpose::STANDARD.encode(&out))
}

/// Decrypt base64(nonce12 || ciphertext_with_tag) into plaintext bytes.
pub fn aes_gcm_decrypt_b64(key: &[u8], b64: &str) -> Result<Vec<u8>, CryptoError> {
  let aes = aes256_from_key_bytes(key)?;
  let combined = general_purpose::STANDARD.decode(b64).map_err(|_| CryptoError::Decode)?;
  if combined.len() < GCM_NONCE_LEN { return Err(CryptoError::CiphertextTooShort); }
  let (nonce_bytes, ct) = combined.split_at(GCM_NONCE_LEN);
  let nonce = Nonce::from_slice(nonce_bytes);
  let pt = aes.decrypt(nonce, Payload { msg: ct, aad: &[] }).map_err(|_| CryptoError::Decrypt)?;
  Ok(pt)
}

/// Convenience: encrypt a UTF-8 string, return base64 payload.
pub fn encrypt_string_b64(key: &[u8], s: &str) -> Result<String, CryptoError> {
  aes_gcm_encrypt_b64(key, s.as_bytes())
}

/// Convenience: decrypt base64 payload to UTF-8 string.
pub fn decrypt_string_b64(key: &[u8], b64: &str) -> Result<String, CryptoError> {
  let pt = aes_gcm_decrypt_b64(key, b64)?;
  let decrypted = String::from_utf8(pt).map_err(|_| CryptoError::Decode)?;
  Ok(decrypted)
}

/// ====== Hex helpers (optional) ======

/// Encode 32-byte key to hex.
pub fn key_bytes_to_hex(key: &[u8]) -> Result<String, CryptoError> {
  if key.len() != 32 { return Err(CryptoError::InvalidLength); }
  Ok(hex::encode(key))
}
pub fn key_hex_to_bytes(hex_key: &str) -> Result<[u8;32], CryptoError> {
  let v = hex::decode(hex_key).map_err(|_| CryptoError::Decode)?;
  if v.len() != 32 { return Err(CryptoError::InvalidLength); }
  let mut out = [0u8;32];
  out.copy_from_slice(&v);
  Ok(out)
}
#[cfg(test)]
mod tests {
  use super::*;
  use p256::elliptic_curve::sec1::ToEncodedPoint;

  #[test]
  fn test_ecdh_derive_same_key() {
    let (sk_a, pub_a_raw) = ecdh_generate_keypair();
    let (sk_b, pub_b_raw) = ecdh_generate_keypair();

    let der_a = export_private_pkcs8_der(&sk_a);
    let der_b = export_private_pkcs8_der(&sk_b);

    let key_ab = derive_aes256_from_ecdh(&der_a, &pub_b_raw).expect("derive A");
    let key_ba = derive_aes256_from_ecdh(&der_b, &pub_a_raw).expect("derive B");

    assert_eq!(key_ab.len(), AES_KEY_LEN);
    assert_eq!(key_ab, key_ba, "Shared keys must match on both sides");
  }

  #[test]
  fn test_encrypt_decrypt_roundtrip_b64() {
    // derive a key
    let (sk_a, _pub_a_raw) = ecdh_generate_keypair();
    let (_sk_b, pub_b_raw) = ecdh_generate_keypair();
    let der_a = export_private_pkcs8_der(&sk_a);
    let key = derive_aes256_from_ecdh(&der_a, &pub_b_raw).expect("derive");

    // round trip
    let payload = "hello world";
    let b64 = aes_gcm_encrypt_b64(&key, payload.as_bytes()).expect("enc");
    let pt = aes_gcm_decrypt_b64(&key, &b64).expect("dec");
    assert_eq!(payload.as_bytes(), pt.as_slice());
  }

  #[test]
  fn test_encrypt_decrypt_box_with_aad() {
    let (sk_a, pub_a_raw) = ecdh_generate_keypair();
    let (sk_b, pub_b_raw) = ecdh_generate_keypair();
    let der_a = export_private_pkcs8_der(&sk_a);
    let der_b = export_private_pkcs8_der(&sk_b);

    let key_ab = derive_aes256_from_ecdh(&der_a, &pub_b_raw).expect("derive A");
    let key_ba = derive_aes256_from_ecdh(&der_b, &pub_a_raw).expect("derive B");
    assert_eq!(key_ab, key_ba);

    let aad = b"session-123";
    let boxed = aes_gcm_encrypt_box_b64(&key_ab, b"secret", Some(aad)).expect("enc");
    // correct AAD
    let pt = aes_gcm_decrypt_box_b64(&key_ba, &boxed, Some(aad)).expect("dec ok");
    assert_eq!(pt, b"secret");

    // wrong AAD should fail
    let bad = aes_gcm_decrypt_box_b64(&key_ba, &boxed, Some(b"wrong"));
    assert!(matches!(bad, Err(CryptoError::Decrypt)));
  }

  #[test]
  fn test_tamper_detected() {
    let (sk, _pub_raw) = ecdh_generate_keypair();
    let der = export_private_pkcs8_der(&sk);
    // derive against our own public just to get a key quickly
    let key = derive_aes256_from_ecdh(&der, &_pub_raw).expect("derive");

    let mut boxed = aes_gcm_encrypt_box_b64(&key, b"data", None).expect("enc");
    // flip one byte in ct
    let mut ct = base64::engine::general_purpose::STANDARD.decode(&boxed.ct).unwrap();
    if !ct.is_empty() { ct[0] ^= 0x01; }
    boxed.ct = base64::engine::general_purpose::STANDARD.encode(ct);

    let res = aes_gcm_decrypt_box_b64(&key, &boxed, None);
    assert!(matches!(res, Err(CryptoError::Decrypt)));
  }

  #[test]
  fn test_pubkey_normalization_and_parsing() {
    // generate a key and compress it to simulate a compressed input
    let (sk, _pub_uncompressed) = ecdh_generate_keypair();
    let pk: PublicKey = sk.public_key();
    let compressed = pk.to_encoded_point(true);
    let b64 = base64::engine::general_purpose::STANDARD.encode(compressed.as_bytes());

    // parse from base64 (compressed) and normalize to uncompressed hex
    let parsed = public_key_from_base64(&b64).expect("parse compressed b64");
    let norm_hex = normalize_pubkey_to_uncompressed_hex(&b64).expect("normalize");

    // ensure normalization length equals 130 hex chars (65 bytes * 2)
    assert_eq!(norm_hex.len(), 130);

    // and converting parsed->uncompressed raw is 65 bytes
    let raw = public_key_to_uncompressed_raw(&parsed);
    assert_eq!(raw.len(), 65);
  }

  #[test]
  fn test_errors() {
    // wrong key length for AES
    let err = aes256_from_key_bytes(&[0u8; 16]);
    assert!(matches!(err, Err(CryptoError::InvalidLength)));

    // bad base64 payload
    let (sk, pub_raw) = ecdh_generate_keypair();
    let der = export_private_pkcs8_der(&sk);
    let key = derive_aes256_from_ecdh(&der, &pub_raw).expect("derive");
    let res = aes_gcm_decrypt_b64(&key, "!!!not-base64!!!");
    assert!(matches!(res, Err(CryptoError::Decode)));
  }
}