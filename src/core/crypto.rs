use anyhow::{Context, Result};
use argon2::{self, Algorithm, Argon2, Params, Version};
use base64::{Engine, engine::general_purpose};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::Aead};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Argon2Config {
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    version: u8,
    salt: String,
    nonce: String,
    argon2: Argon2Config,
}

#[derive(Error, Debug)]
pub enum EnvLockError {
    #[error("Password mismatch or corrupted data")]
    DecryptionError,
}

pub fn encrypt_env(plaintext: &str, password: &str) -> Result<(Vec<u8>, Meta)> {
    // Parameters
    let m_cost = 65536; // 64 MB
    let t_cost = 3;
    let p_cost = 1;

    // Random salt & nonce
    let mut salt = [0u8; 16];
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut nonce_bytes);

    let params = Params::new(m_cost, t_cost, p_cost, None).expect("Invalid Argon2 parameters");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key_bytes = [0u8; 32];
    let _ = argon2
        .hash_password_into(password.as_bytes(), &salt, &mut key_bytes)
        .map_err(|e| anyhow::anyhow!("Key derivation failed: {e}"));

    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .context("Encryption failed")?;

    let meta = Meta {
        version: 1,
        salt: general_purpose::STANDARD.encode(salt),
        nonce: general_purpose::STANDARD.encode(nonce_bytes),
        argon2: Argon2Config {
            m_cost,
            t_cost,
            p_cost,
        },
    };

    Ok((ciphertext, meta))
}

pub fn decrypt_env(ciphertext: &[u8], password: &str, meta: &Meta) -> Result<String> {
    let salt = general_purpose::STANDARD
        .decode(&meta.salt)
        .context("Invalid base64 salt")?;
    let nonce_bytes = general_purpose::STANDARD
        .decode(&meta.nonce)
        .context("Invalid base64 nonce")?;

    let params = Params::new(
        meta.argon2.m_cost,
        meta.argon2.t_cost,
        meta.argon2.p_cost,
        None,
    )
    .expect("Invalid Argon2 parameters");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key_bytes = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), &salt, &mut key_bytes)
        .map_err(|e| anyhow::anyhow!("Key derivation failed: {e}"))?;

    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| EnvLockError::DecryptionError)?;

    let plaintext =
        String::from_utf8(decrypted_bytes).context("Decrypted data is not valid UTF-8")?;
    Ok(plaintext)
}
