use crate::error::CustomError;
use anyhow::Result;

use super::model::User;

pub struct AuthRepository {
    pool: sqlx::AnyPool,
}

impl AuthRepository {
    pub fn new(pool: sqlx::AnyPool) -> Self {
        AuthRepository { pool }
    }

    pub fn print_pool_stats(&self) {
        println!("[DB POOL STATS]");
        println!("Total connections: {}", self.pool.size());
        println!("Idle connections: {}", self.pool.num_idle());
        println!(
            "Active connections: {}",
            self.pool.size() - self.pool.num_idle() as u32
        );
    }

    pub async fn query_user(&self, user_login: &User) -> Result<User, CustomError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password 
            FROM users 
            WHERE username = $1
            "#,
        )
        .bind(&user_login.username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => CustomError::UserNotFound,
            _ => CustomError::DBError(e),
        })?;

        Ok(user)
    }

    pub async fn insert_user(&self, new_user: &User) -> Result<u64, CustomError> {
        let row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO users (username, password) 
            VALUES ($1, $2) 
            RETURNING id"#,
        )
        .bind(&new_user.username)
        .bind(&new_user.password)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(err) if err.is_unique_violation() => CustomError::UsernameExists,
            e => CustomError::DBError(e),
        })?;

        Ok(row.0 as u64)
    }
}
