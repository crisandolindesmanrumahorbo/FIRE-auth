use crate::auth::Claims;
use crate::auth::User;
use crate::config::CONFIG;
use crate::error::CustomError;
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

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
