use anyhow::Error;
use block_modes::BlockMode;
use p256::elliptic_curve;
use p256::elliptic_curve::sec1::FromEncodedPoint;
use p256::pkcs8::FromPrivateKey;
use p256::{ecdh::EphemeralSecret, EncodedPoint};
use rand_core::OsRng;
use spki::der::Decodable;
use spki::der::Encodable;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use tendril::fmt::Slice;
use crate::error::{FastJobErrorType, FastJobResult};

#[derive(Debug, Clone, Default)]
pub struct Crypto {
  secret_key: Vec<u8>,
  iv: Vec<u8>,
}

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
  // ECDH
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
    Self {
      0: vec,
    }
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

impl From<(Vec<u8>, Vec<u8>)> for Crypto {
  fn from((secret_key, iv): (Vec<u8>, Vec<u8>)) -> Self {
    Crypto { secret_key, iv }
  }
}

impl Crypto {
  pub fn new(secret_key: Vec<u8>, iv: Vec<u8>) -> Self {
    Self { secret_key, iv }
  }

  pub fn encrypt_aes_cbc(&self, length: usize, data: DataBuffer) -> Result<String, AnyError> {
    let key = &self.secret_key;
    let iv = &self.iv;

    let ciphertext = match length {
      128 => {
        // Section 10.3 Step 2 of RFC 2315 https://www.rfc-editor.org/rfc/rfc2315
        type Aes128Cbc = block_modes::Cbc<aes::Aes128, block_modes::block_padding::Pkcs7>;

        let cipher = Aes128Cbc::new_from_slices(&key, &iv)?;
        cipher.encrypt_vec(&data.buf.0)
      }
      192 => {
        // Section 10.3 Step 2 of RFC 2315 https://www.rfc-editor.org/rfc/rfc2315
        type Aes192Cbc = block_modes::Cbc<aes::Aes192, block_modes::block_padding::Pkcs7>;

        let cipher = Aes192Cbc::new_from_slices(&key, &iv)?;
        cipher.encrypt_vec(&data.buf.0)
      }
      256 => {
        // Section 10.3 Step 2 of RFC 2315 https://www.rfc-editor.org/rfc/rfc2315
        type Aes256Cbc = block_modes::Cbc<aes::Aes256, block_modes::block_padding::Pkcs7>;

        let cipher = Aes256Cbc::new_from_slices(&key, &iv)?;
        cipher.encrypt_vec(&data.buf.0)
      }
      _ => return Err(type_error("invalid length")),
    };
    let res = BASE64_STANDARD.encode(ciphertext);
    Ok(res)
  }

  pub fn decrypt_aes_cbc(&self, length: usize, data: DataBuffer) -> Result<Vec<u8>, AnyError> {
    let key = &self.secret_key;
    let iv = &self.iv;
    let plaintext = match length {
      128 => {
        // Section 10.3 Step 2 of RFC 2315 https://www.rfc-editor.org/rfc/rfc2315
        type Aes128Cbc = block_modes::Cbc<aes::Aes128, block_modes::block_padding::Pkcs7>;
        let cipher = Aes128Cbc::new_from_slices(&key, &iv)?;

        cipher
         .decrypt_vec(&data.buf.0)
         .map_err(|_| data_error("invalid data"))?
      }
      192 => {
        // Section 10.3 Step 2 of RFC 2315 https://www.rfc-editor.org/rfc/rfc2315
        type Aes192Cbc = block_modes::Cbc<aes::Aes192, block_modes::block_padding::Pkcs7>;
        let cipher = Aes192Cbc::new_from_slices(&key, &iv)?;

        cipher
         .decrypt_vec(&data.buf.0)
         .map_err(|_| data_error("invalid data"))?
      }
      256 => {
        // Section 10.3 Step 2 of RFC 2315 https://www.rfc-editor.org/rfc/rfc2315
        type Aes256Cbc = block_modes::Cbc<aes::Aes256, block_modes::block_padding::Pkcs7>;
        let cipher = Aes256Cbc::new_from_slices(&key, &iv)?;

        cipher
         .decrypt_vec(&data.buf.0)
         .map_err(|_| data_error("invalid data"))?
      }
      _ => unreachable!(),
    };

    // 6.
    //info!("PLAINTEXT VEC {:?} ",&plaintext);
    Ok(plaintext)
  }

  pub fn import_public_key(data: DataBuffer) -> Result<Vec<u8>, AnyError> {
    // 2-3.
    let pk_info = spki::SubjectPublicKeyInfo::from_der(&data.buf.0)
     .map_err(|e| data_error(e.to_string()))?;
    // 4.
    let alg = pk_info.algorithm.oid;
    // id-ecPublicKey
    if alg != elliptic_curve::ALGORITHM_OID {
      return Err(data_error("unsupported algorithm"));
    }
    let pk = pk_info.subject_public_key;

    let encoded_key = pk.to_vec();
    Ok(encoded_key.to_vec().into())
  }
  pub fn export_public_key(data_buf: DataBuffer) -> Result<Vec<u8>, AnyError> {
    let point = data_buf.buf;

    let subject_public_key = point.0.as_bytes();
    let alg_id =
     <p256::NistP256 as p256::elliptic_curve::AlgorithmParameters>::algorithm_identifier();

    // the SPKI structure
    let key_info = spki::SubjectPublicKeyInfo {
      algorithm: alg_id,
      subject_public_key: &subject_public_key,
    };
    let spki_der = key_info.to_vec().map_err(|_| data_error("Failed to encode SPKI"))?;

    Ok(spki_der.into())
  }

  pub fn generate_key() -> FastJobResult<(EphemeralSecret, Vec<u8>)> {

    let secret = EphemeralSecret::random(&mut OsRng);
    let pk_bytes = EncodedPoint::from(secret.public_key());
    Ok((secret, pk_bytes.as_ref().to_vec()))
  }
  pub fn derive_key(args: DeriveKeyArg) -> FastJobResult<Vec<u8>> {
    let public_key = args
     .public_key
     .ok_or_else(|| type_error("Missing argument publicKey"))?;
    let secret_key = p256::SecretKey::from_pkcs8_der(args.key.data.buf.0.as_bytes())
     .map_err(|_| data_error("Unexpected error decoding private key"))?;
    //info!("SSSS");
    let public_key = match public_key.r#type {
      KeyType::Private => p256::SecretKey::from_pkcs8_der(&public_key.data.buf.0)
       .map_err(|_| type_error("Unexpected error decoding private key"))?
       .public_key(),
      KeyType::Public => {
        let point = p256::EncodedPoint::from_bytes(public_key.data.buf.0)
         .map_err(|_| type_error("Unexpected error decoding private key"))?;

        p256::PublicKey::from_encoded_point(&point)
         .ok_or_else(|| type_error("Unexpected error decoding private key"))?

      }
      _ => unreachable!(),
    };
    let shared_secret = p256::elliptic_curve::ecdh::diffie_hellman(
      secret_key.to_secret_scalar(),
      public_key.as_affine(),
    );

    Ok(shared_secret.as_bytes().to_vec().into())
  }

  pub fn derive_secret_key(
    private_key: KeyData,
    public_key: Option<KeyData>,
  ) -> FastJobResult<String> {
    let args = DeriveKeyArg {
      key: private_key,
      public_key,
    };
    let secure_key = Crypto::derive_key(args)?;
    let res = hex::encode(&secure_key);
    Ok(res)
  }
}

pub fn xchange_decrypt_data(
  encrypted_data: String,
  hex_secret_key: String,
  session: &str,
) -> FastJobResult<String> {
  let iv = session[5..21].to_string();

  let secret_key = hex::decode(hex_secret_key).map_err(|_| FastJobErrorType::DecryptingError)?;
  let crypto = Crypto::from((secret_key, iv.as_bytes().to_vec()));

  let data = BASE64_STANDARD
   .decode(&encrypted_data)
   .map_err(|_| FastJobErrorType::DecodeError)?;

  match crypto.decrypt_aes_cbc(256, DataBuffer::from_vec(&data)) {
    Ok(decrypt_vec) => {
      let decrypted_data = String::from_utf8(decrypt_vec.clone())?;
      Ok(decrypted_data)
    }
    Err(_err) => Err(FastJobErrorType::DecryptingError.into()),
  }
}
pub fn xchange_encrypt_data(
  data: String,
  hex_secret_key: String,
  session: String,
) -> FastJobResult<String> {
  let iv = session[5..21].to_string();

  let secret_key = hex::decode(hex_secret_key).map_err(|_| FastJobErrorType::EncryptingError)?;
  let crypto = Crypto::from((secret_key, iv.as_bytes().to_vec()));

  match crypto.encrypt_aes_cbc(256, DataBuffer::from_vec(data.as_bytes())) {
    Ok(encrypted_text) => Ok(encrypted_text),
    Err(_err) => Err(FastJobErrorType::EncryptingError.into()),
  }
}
pub type AnyError = anyhow::Error;

#[derive(Debug)]
struct CustomError {
  message: Cow<'static, str>,
}

impl Display for CustomError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(&self.message)
  }
}

impl std::error::Error for CustomError {}

pub fn custom_error(_class: &'static str, message: impl Into<Cow<'static, str>>) -> Error {
  CustomError {
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