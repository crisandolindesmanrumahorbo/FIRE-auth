use dotenvy::from_filename;
use sqlx::{AnyPool, any::install_default_drivers};
use std::env;

pub async fn setup_test_db() -> AnyPool {
    from_filename(".env.test").ok();
    let env_db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    install_default_drivers();
    let pool = AnyPool::connect(&env_db_url)
        .await
        .expect("Failed to create in-memory SQLite DB");

    sqlx::query("DROP TABLE IF EXISTS users;")
        .execute(&pool)
        .await
        .expect("Failed to drop test table");

    sqlx::query(
        "
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL
        );
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create test table");

    println!("✅ Test database setup complete.");
    pool
}
