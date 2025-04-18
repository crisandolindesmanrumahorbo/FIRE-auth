use rand::Rng;
use sqlx::{AnyPool, any::install_default_drivers};

pub async fn setup_test_db() -> AnyPool {
    // init config
    stockbit_auth::cfg::init_config();

    // init db
    install_default_drivers();
    let rand_str: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let db_name = format!("test_{}", rand_str);
    let database_url = format!("sqlite:file:{}?mode=memory&cache=shared", db_name);
    // Create the pool (which will internally use shared memory DB)
    let pool = AnyPool::connect(&database_url)
        .await
        .expect("Failed to create in-memory SQLite DB");

    // create table
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

    println!("✅ Pool created with unique DB: {}", db_name);

    pool
}

pub async fn insert_db_user(username: &str, password: &str, pool: &AnyPool) {
    sqlx::query(
        r#"
            INSERT INTO users (username, password) 
            VALUES ($1, $2) 
            RETURNING id"#,
    )
    .bind(username)
    .bind(password)
    .execute(pool)
    .await
    .expect("Failed to insert test user");
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
