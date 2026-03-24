use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use std::io;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

fn get_key() -> [u8; 16] {
    [
        0x52, 0x65, 0x64, 0x69, 0x73, 0x44, 0x65, 0x73, 0x6b, 0x74, 0x6f, 0x70, 0x53, 0x65, 0x63,
        0x31,
    ]
}

#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub iv: String,
}

pub fn encrypt_password(plaintext: &str) -> io::Result<EncryptedData> {
    if plaintext.is_empty() {
        return Ok(EncryptedData {
            ciphertext: String::new(),
            iv: String::new(),
        });
    }

    let key = get_key();
    let iv_bytes: [u8; 16] = rand::rng().random();
    let iv = general_purpose::STANDARD.encode(iv_bytes);

    let plaintext_bytes = plaintext.as_bytes();
    let padded_len = plaintext_bytes.len() + 16 - (plaintext_bytes.len() % 16);
    let mut buffer = vec![0u8; padded_len];
    buffer[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);

    let cipher = Aes128CbcEnc::new((&key).into(), (&iv_bytes).into());
    let ciphertext_bytes = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, plaintext_bytes.len())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let ciphertext = general_purpose::STANDARD.encode(ciphertext_bytes);

    Ok(EncryptedData { ciphertext, iv })
}

pub fn decrypt_password(ciphertext: &str, iv: &str) -> io::Result<String> {
    if ciphertext.is_empty() || iv.is_empty() {
        return Ok(String::new());
    }

    let key = get_key();
    let mut ciphertext_bytes = general_purpose::STANDARD
        .decode(ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let iv_bytes: [u8; 16] = general_purpose::STANDARD
        .decode(iv)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid IV length"))?;

    let cipher = Aes128CbcDec::new((&key).into(), (&iv_bytes).into());
    let decrypted_bytes = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut ciphertext_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    String::from_utf8(decrypted_bytes.to_vec())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurePassword {
    pub encrypted: String,
    pub iv: String,
    #[serde(default)]
    pub is_encrypted: bool,
}

impl Default for SecurePassword {
    fn default() -> Self {
        Self {
            encrypted: String::new(),
            iv: String::new(),
            is_encrypted: false,
        }
    }
}

impl SecurePassword {
    pub fn from_plaintext(plaintext: &str) -> io::Result<Self> {
        if plaintext.is_empty() {
            return Ok(Self::default());
        }

        let encrypted = encrypt_password(plaintext)?;
        Ok(Self {
            encrypted: encrypted.ciphertext,
            iv: encrypted.iv,
            is_encrypted: true,
        })
    }

    pub fn decrypt(&self) -> io::Result<String> {
        if !self.is_encrypted {
            return Ok(self.encrypted.clone());
        }
        decrypt_password(&self.encrypted, &self.iv)
    }

    pub fn is_empty(&self) -> bool {
        self.encrypted.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_password() {
        let plaintext = "my_secret_password_123";
        let secure = SecurePassword::from_plaintext(plaintext).unwrap();

        assert!(secure.is_encrypted);
        assert_ne!(secure.encrypted, plaintext);

        let decrypted = secure.decrypt().unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_empty_password() {
        let secure = SecurePassword::from_plaintext("").unwrap();
        assert!(!secure.is_encrypted);
        assert!(secure.is_empty());

        let decrypted = secure.decrypt().unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_password_with_special_chars() {
        let plaintext = "p@ssw0rd!#$%^&*()_+-={}[]|:;<>?,./~`";
        let secure = SecurePassword::from_plaintext(plaintext).unwrap();
        let decrypted = secure.decrypt().unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_password_with_unicode() {
        let plaintext = "密码测试123🔐";
        let secure = SecurePassword::from_plaintext(plaintext).unwrap();
        let decrypted = secure.decrypt().unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
