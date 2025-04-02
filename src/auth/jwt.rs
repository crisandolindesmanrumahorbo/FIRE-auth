use crate::auth::Claims;
use crate::auth::User;
use crate::error::CustomError;
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{EncodingKey, Header, encode};
use std::env;

pub fn get_private_key() -> Result<EncodingKey, CustomError> {
    dotenv().ok();
    let key = env::var("JWT_PRIVATE_KEY")
        .map_err(|e| CustomError::EnvError("JWT_PRIVATE_KEY".to_string(), e))?;
    let enc_key = EncodingKey::from_rsa_pem(key.replace("\\n", "\n").as_bytes())
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
