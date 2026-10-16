//! Password hashing and verification (Argon2id).
//!
//! Hashing is CPU/memory heavy, so it runs on `spawn_blocking` to keep it off
//! the async reactor. Hashes are PHC strings; no plaintext is ever stored.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::AppError;

/// Hash a password into a PHC string using Argon2id with default (OWASP-ish)
/// parameters. Parameters live in `Argon2::default()` and can be upgraded later
/// without a schema change since the cost is encoded in the stored hash.
pub async fn hash(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    })
    .await
    .map_err(AppError::internal)?
    .map_err(AppError::from)
}

/// Verify a candidate password against a stored PHC hash.
pub async fn verify(password: String, phc: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || match PasswordHash::new(&phc) {
        Ok(parsed) => Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()),
        Err(err) => Err(err),
    })
    .await
    .map_err(AppError::internal)?
    .map_err(AppError::from)
}
