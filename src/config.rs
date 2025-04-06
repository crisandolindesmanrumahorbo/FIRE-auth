use std::env;

use dotenvy::dotenv;
use once_cell::sync::Lazy;

pub struct Config {
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    pub database_url: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv().ok(); // Load environment variables

    Config {
        jwt_private_key: env::var("JWT_PRIVATE_KEY").expect("JWT_PRIVATE_KEY must be set"),
        jwt_public_key: env::var("JWT_PUBLIC_KEY").expect("JWT_PUBLIC_KEY must be set"),
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    }
});

pub fn init_config() -> &'static Config {
    &CONFIG
}
