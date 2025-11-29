// === AES-256-GCM encryption for credential storage ===

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::Rng;

use crate::error::GatewayError;

const NONCE_SIZE: usize = 12;

/// Encrypt plaintext using AES-256-GCM
#[allow(deprecated)]
pub fn encrypt(plaintext: &str, key: &str) -> Result<String, GatewayError> {
    let key_bytes = derive_key(key);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| GatewayError::Internal(format!("Cipher init failed: {}", e)))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| GatewayError::Internal(format!("Encryption failed: {}", e)))?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);

    Ok(STANDARD.encode(result))
}

/// Decrypt ciphertext using AES-256-GCM
#[allow(deprecated)]
pub fn decrypt(encrypted: &str, key: &str) -> Result<String, GatewayError> {
    let key_bytes = derive_key(key);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| GatewayError::Internal(format!("Cipher init failed: {}", e)))?;

    let data = STANDARD
        .decode(encrypted)
        .map_err(|e| GatewayError::Internal(format!("Base64 decode failed: {}", e)))?;

    if data.len() < NONCE_SIZE {
        return Err(GatewayError::Internal("Invalid encrypted data".to_string()));
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| GatewayError::Internal(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| GatewayError::Internal(format!("UTF-8 decode failed: {}", e)))
}

/// Derive 32-byte key from password using simple padding
/// Note: In production, use a proper KDF like Argon2 or PBKDF2
fn derive_key(password: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    let bytes = password.as_bytes();
    for (i, byte) in bytes.iter().cycle().take(32).enumerate() {
        key[i] = *byte;
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = "my-secret-key-32-chars-long!!!!";
        let plaintext = "secret_token_12345";

        let encrypted = encrypt(plaintext, key).unwrap();
        let decrypted = decrypt(&encrypted, key).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
