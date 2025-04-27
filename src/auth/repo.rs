use async_trait::async_trait;

use crate::error::CustomError;

use super::model::User;

#[async_trait]
pub trait DbConnection: Send + Sync {
    async fn fetch_user(&self, username: &str) -> Result<User, sqlx::Error>;
    async fn insert_user(&self, user: &User) -> Result<u64, sqlx::Error>;
    fn print_pool_stats(&self);
}

#[async_trait]
impl DbConnection for sqlx::PgPool {
    async fn fetch_user(&self, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(r#"SELECT id, username, password FROM users WHERE username = $1"#)
            .bind(username)
            .fetch_one(self)
            .await
    }
    async fn insert_user(&self, user: &User) -> Result<u64, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO users (username, password) 
            VALUES ($1, $2) 
            RETURNING id"#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .fetch_one(self)
        .await?;
        Ok(row.0 as u64)
    }
    fn print_pool_stats(&self) {
        println!("[DB POOL STATS]");
        println!("Total connections: {}", self.size());
        println!("Idle connections: {}", self.num_idle());
        println!(
            "Active connections: {}",
            self.size() - self.num_idle() as u32
        );
    }
}

#[cfg(feature = "test-sqlite")]
#[async_trait]
impl DbConnection for sqlx::SqlitePool {
    async fn fetch_user(&self, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(r#"SELECT id, username, password FROM users WHERE username = ?1"#)
            .bind(username)
            .fetch_one(self)
            .await
    }
    async fn insert_user(&self, user: &User) -> Result<u64, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO users (username, password)
            VALUES ($1, $2)
            RETURNING id"#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .fetch_one(self)
        .await?;
        Ok(row.0 as u64)
    }
    fn print_pool_stats(&self) {
        println!("[DB POOL STATS]");
        println!("Total connections: {}", self.size());
        println!("Idle connections: {}", self.num_idle());
        println!(
            "Active connections: {}",
            self.size() - self.num_idle() as u32
        );
    }
}

pub struct AuthRepository<DB: DbConnection> {
    db: DB,
}

impl<DB: DbConnection> AuthRepository<DB> {
    pub fn new(db: DB) -> Self {
        AuthRepository { db }
    }

    pub fn print_pool_stats(&self) {
        self.db.print_pool_stats();
    }

    pub async fn query_user(&self, user_login: &User) -> Result<User, CustomError> {
        self.db
            .fetch_user(&user_login.username)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => CustomError::UserNotFound,
                _ => CustomError::DBError(e),
            })
    }

    pub async fn insert_user(&self, new_user: &User) -> Result<u64, CustomError> {
        self.db.insert_user(new_user).await.map_err(|e| match e {
            sqlx::Error::Database(err) if err.is_unique_violation() => CustomError::UsernameExists,
            e => CustomError::DBError(e),
        })
    }
}
