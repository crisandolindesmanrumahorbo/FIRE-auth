use bcrypt::{DEFAULT_COST, hash, verify};
use serde::{Deserialize, Serialize};
use auth::model::Claims;
use auth::model::User;
use crate::auth;
use crate::config::CONFIG;
use crate::error::CustomError;
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

pub fn des_from_str<T: for<'a> Deserialize<'a> + Serialize>(string: &str) -> Result<T, ()> {
    serde_json::from_str(&string.split("\r\n\r\n").last().unwrap_or_default()).map_err(|_| ())
}

pub fn ser_to_str<T: for<'a> Deserialize<'a> + Serialize>(
    t: &T,
) -> Result<String, serde_json::Error> {
    Ok(serde_json::to_string(t)?)
}

pub fn encrypt(value: &str) -> String {
    hash(value, DEFAULT_COST).expect("generate password failed")
}

pub fn compare(value: &str, value1: &str) -> bool {
    verify(value, value1).unwrap_or(false)
}


fn get_private_key() -> Result<EncodingKey, CustomError> {
    let enc_key = EncodingKey::from_rsa_pem(CONFIG.jwt_private_key.replace("\\n", "\n").as_bytes())
        .map_err(|e| CustomError::EncodeError(e))?;
    Ok(enc_key)
}

pub fn create_jwt(user: User) -> Result<String> {
    let private_key = get_private_key().context("Failed Get Private Key")?;
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24)) // Token valid for 24 hours
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.username,
        exp: expiration,
    };

    encode(
        &Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &private_key,
    )
    .context("Failed to Encode the JWT")
}
