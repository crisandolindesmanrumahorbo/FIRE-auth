use std::sync::atomic::{AtomicPtr, Ordering};
use std::{env, ptr};

pub struct Config {
    pub jwt_private_key: &'static str,
    pub jwt_public_key: &'static str,
    pub database_url: &'static str,
}

static CONFIG: AtomicPtr<Config> = AtomicPtr::new(ptr::null_mut());

pub fn init_config() {
    dotenvy::dotenv().ok();
    let config = Box::new(Config {
        jwt_private_key: Box::leak(
            env::var("JWT_PRIVATE_KEY")
                .expect("JWT_PRIVATE_KEY must be set")
                .into_boxed_str(),
        ),
        jwt_public_key: Box::leak(
            env::var("JWT_PUBLIC_KEY")
                .expect("JWT_PUBLIC_KEY must be set")
                .into_boxed_str(),
        ),
        database_url: Box::leak(
            env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set")
                .into_boxed_str(),
        ),
    });
    CONFIG.store(Box::into_raw(config), Ordering::Release);
}

#[inline]
pub fn get_config() -> &'static Config {
    // SAFETY: Initialized at startup before any threads
    unsafe { &*CONFIG.load(Ordering::Acquire) }
}
