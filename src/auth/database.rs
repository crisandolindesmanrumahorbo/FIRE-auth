use crate::{auth::model::User, error::CustomError};
use dotenvy::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions, query_as};
use std::env;
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

async fn init_db_pool() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(5)
        .idle_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await
        .expect("Failed to create DB pool")
}

pub async fn get_db_pool() -> &'static PgPool {
    DB_POOL.get_or_init(|| async { init_db_pool().await }).await
}

pub async fn print_pool_stats(pool: &PgPool) {
    println!("[DB POOL STATS]");
    println!("Total connections: {}", pool.size());
    println!("Idle connections: {}", pool.num_idle());
    println!("Active connections: {}", pool.size() - pool.num_idle() as u32);
}

pub async fn query_user(user_login: &User) -> Result<User, CustomError> {
    let pool = get_db_pool().await;

    let user = query_as::<_, User>(
        r#"
        SELECT id, username, password 
        FROM users 
        WHERE username = $1
        "#,
    )
    .bind(&user_login.username)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => CustomError::UserNotFound,
        _ => CustomError::DBError(e),
    })?;

    Ok(user)
}

pub async fn insert_user(new_user: &User) -> Result<u64, CustomError> {
    let pool = get_db_pool().await;
    let row: (i32,) = sqlx::query_as(
        r#"
        INSERT INTO users (username, password) 
        VALUES ($1, $2) 
        RETURNING id"#,
    )
    .bind(&new_user.username)
    .bind(&new_user.password)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(err) if err.is_unique_violation() => CustomError::UsernameExists,
        e => CustomError::DBError(e),
    })?;

    Ok(row.0 as u64)
}
