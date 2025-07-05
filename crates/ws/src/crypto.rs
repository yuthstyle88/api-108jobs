use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::{rngs::OsRng, RngCore};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

#[derive(Clone)]
pub struct Crypto {
  key: [u8; 32],
  iv: [u8; 16],
}

impl Crypto {
  pub fn new() -> Self {
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];

    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut iv);

    Self { key, iv }
  }

  pub fn get_key_hex(&self) -> String {
    hex::encode(self.key)
  }

  pub fn get_iv_hex(&self) -> String {
    hex::encode(self.iv)
  }

  pub fn encrypt(&self, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let cipher = Aes256CbcEnc::new(&self.key.into(), &self.iv.into());
    let encrypted = cipher.encrypt_padded_vec_mut::<Pkcs7>(data);
    Ok(hex::encode(encrypted))
  }

  pub fn decrypt(&self, hex_data: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let encrypted = hex::decode(hex_data)?;
    let cipher = Aes256CbcDec::new(&self.key.into(), &self.iv.into());
    let decrypted = cipher.decrypt_padded_vec_mut::<Pkcs7>(&encrypted)?;
    Ok(decrypted)
  }
}