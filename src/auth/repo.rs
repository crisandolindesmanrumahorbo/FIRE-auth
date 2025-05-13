use async_trait::async_trait;

use crate::{account::model::Account, error::CustomError};

use super::model::User;

#[async_trait]
pub trait DbConnection: Send + Sync {
    async fn fetch_user(&self, username: &str) -> Result<User, sqlx::Error>;
    async fn insert_user(&self, user: &User) -> Result<i32, sqlx::Error>;
    async fn insert_account(&self, account: &Account) -> Result<i32, sqlx::Error>;
    fn print_pool_stats(&self);
}

#[async_trait]
impl DbConnection for sqlx::PgPool {
    async fn fetch_user(&self, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"SELECT user_id, username, password, created_at FROM users WHERE username = $1"#,
        )
        .bind(username)
        .fetch_one(self)
        .await
    }
    async fn insert_user(&self, user: &User) -> Result<i32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO users (username, password, created_at) 
            VALUES ($1, $2, $3) 
            RETURNING user_id"#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.created_at)
        .fetch_one(self)
        .await?;
        Ok(row.0)
    }
    async fn insert_account(&self, account: &Account) -> Result<i32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO accounts (user_id, balance, invested_value) 
            VALUES ($1, $2, $3, $4) 
            RETURNING account_id"#,
        )
        .bind(&account.user_id)
        .bind(&account.balance)
        .bind(&account.invested_value)
        .fetch_one(self)
        .await?;
        Ok(row.0)
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
        sqlx::query_as::<_, User>(
            r#"SELECT user_id, username, password, created_at FROM users WHERE username = ?1"#,
        )
        .bind(username)
        .fetch_one(self)
        .await
    }
    async fn insert_user(&self, user: &User) -> Result<i32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO users (username, password, created_at)
            VALUES ($1, $2, $3)
            RETURNING user_id"#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.created_at)
        .fetch_one(self)
        .await?;
        Ok(row.0)
    }
    async fn insert_account(&self, account: &Account) -> Result<i32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO accounts (user_id, balance, invested_value) 
            VALUES ($1, $2, $3, $4) 
            RETURNING account_id"#,
        )
        .bind(&account.user_id)
        .bind(&account.balance)
        .bind(&account.invested_value)
        .fetch_one(self)
        .await?;
        Ok(row.0)
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

    pub async fn query_user(&self, username: &str) -> Result<User, CustomError> {
        self.db.fetch_user(username).await.map_err(|e| match e {
            sqlx::Error::RowNotFound => CustomError::UserNotFound,
            _ => CustomError::DBError(e),
        })
    }

    pub async fn insert_user(&self, new_user: &User) -> Result<i32, CustomError> {
        let user_id = match self.db.insert_user(new_user).await {
            Ok(user_id) => user_id,
            Err(e) => match e {
                sqlx::Error::Database(err) if err.is_unique_violation() => {
                    return Err(CustomError::UsernameExists);
                }
                _ => return Err(CustomError::DBError(e)),
            },
        };
        let new_account = Account::new(user_id);
        match self.db.insert_account(&new_account).await {
            Ok(user_id) => user_id,
            Err(e) => match e {
                sqlx::Error::Database(err) if err.is_unique_violation() => {
                    return Err(CustomError::AccountExists);
                }
                _ => return Err(CustomError::DBError(e)),
            },
        };
        Ok(user_id)
    }
}
