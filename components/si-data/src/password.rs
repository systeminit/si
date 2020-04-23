use crate::error::{DataError, Result};
use sodiumoxide::crypto::pwhash::argon2id13;

pub fn encrypt_password(input_password: Option<String>) -> Result<String> {
    if input_password.is_none() {
        return Err(DataError::EmptyPassword);
    }
    let password = input_password.unwrap();
    let password_hash = argon2id13::pwhash(
        password.as_bytes(),
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE,
    )
    .map_err(|_| DataError::PasswordHash)?;
    let password_hash_str = std::str::from_utf8(password_hash.as_ref())?;
    Ok(password_hash_str.to_string())
}

pub fn verify_password(password: &str, password_hash: String) -> bool {
    let password_bytes = password.as_bytes();
    if let Some(argon_password) = argon2id13::HashedPassword::from_slice(password_hash.as_bytes()) {
        if argon2id13::pwhash_verify(&argon_password, password_bytes) {
            true
        } else {
            false
        }
    } else {
        false
    }
}
