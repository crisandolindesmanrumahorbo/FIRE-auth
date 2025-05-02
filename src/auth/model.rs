use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct User {
    pub user_id: Option<i32>,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}
