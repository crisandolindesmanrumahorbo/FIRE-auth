use crate::auth::Claims;
use crate::auth::User;
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{EncodingKey, Header, encode};
use std::env;

pub fn get_private_key() -> Result<EncodingKey, &'static str> {
    dotenv().ok();
    let key = match env::var("JWT_PRIVATE_KEY") {
        Ok(key) => key,
        Err(_) => {
            return Err("Missing JWT_PRIVATE_KEY");
        }
    };
    let enc_key = match EncodingKey::from_rsa_pem(key.replace("\\n", "\n").as_bytes()) {
        Ok(key) => key,
        Err(_) => {
            return Err("Encoding Key error");
        }
    };
    Ok(enc_key)
}

pub fn create_jwt(user: User) -> Result<String, &'static str> {
    let private_key = get_private_key()?;
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24)) // Token valid for 24 hours
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.username,
        exp: expiration,
    };

    let encoding_key = match encode(
        &Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &private_key,
    ) {
        Ok(ek) => ek,
        Err(_) => return Err("Encode Error"),
    };
    Ok(encoding_key)
}