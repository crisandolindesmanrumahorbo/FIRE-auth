use sqlx::Pool;

pub struct Database {
    pub pool: Pool<sqlx::Any>,
}

impl Database {
    pub async fn new_pool(url: &str) -> Pool<sqlx::Any> {
        sqlx::any::install_default_drivers();
        sqlx::any::AnyPoolOptions::new()
            .max_connections(10)
            .min_connections(5)
            .idle_timeout(std::time::Duration::from_secs(30))
            .connect(url)
            .await
            .expect("Failed to create DB pool")
    }
}
