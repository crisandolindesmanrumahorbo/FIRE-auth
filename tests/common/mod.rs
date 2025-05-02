use chrono::{DateTime, Utc};
use rand::Rng;
use sqlx::SqlitePool;
use stockbit_auth::cfg::init_config;

pub async fn setup_test_db() -> SqlitePool {
    // init config
    init_config();
    // init db
    let rand_str: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let db_name = format!("test_{}", rand_str);
    let database_url = format!("sqlite:file:{}?mode=memory&cache=shared", db_name);
    // Create the pool (which will internally use shared memory DB)
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create in-memory SQLite DB");

    // create table
    sqlx::query(
        "
        CREATE TABLE users (
            user_id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create test table");

    println!("✅ Pool created with unique DB: {}", db_name);

    pool
}

pub async fn insert_db_user(
    username: &str,
    password: &str,
    created_at: DateTime<Utc>,
    pool: &SqlitePool,
) {
    let _row: (i32,) = sqlx::query_as(
        r#"
            INSERT INTO users (username, password, created_at)
            VALUES ($1, $2, $3)
            RETURNING user_id"#,
    )
    .bind(username)
    .bind(password)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .unwrap();
}

pub struct TestWriter<'a>(pub &'a mut Vec<u8>);
impl tokio::io::AsyncWrite for TestWriter<'_> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.0.extend_from_slice(buf);
        std::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}
