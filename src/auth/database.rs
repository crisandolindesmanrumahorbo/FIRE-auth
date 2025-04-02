use std::env::{self, VarError};

use crate::{auth::model::User, error::CustomError};
use dotenvy::dotenv;
use postgres::{Client, NoTls};

pub fn check_user_db(user_login: &User) -> Result<User, CustomError> {
    let url = get_db_url().map_err(|e| CustomError::EnvError("DATABASE_URL".to_string(), e))?;
    let mut client = Client::connect(&url, NoTls).map_err(|e| CustomError::DBConnectionError(e))?;
    let row = client
        .query_one(
            "SELECT * FROM users WHERE username = $1",
            &[&user_login.username],
        )
        .map_err(|e| CustomError::DBQueryError(e))?;

    let user = User {
        id: row.get(0),
        username: row.get(1),
        password: row.get(2),
    };
    Ok(user)
}

pub fn insert_db_user(new_user: &User) -> Result<u64, CustomError> {
    let url = get_db_url().map_err(|e| CustomError::EnvError("DATABASE_URL".to_string(), e))?;
    let mut client = Client::connect(&url, NoTls).map_err(|e| CustomError::DBConnectionError(e))?;
    Ok(client
        .execute(
            "INSERT INTO users (username, password) VALUES ($1, $2)",
            &[&new_user.username, &new_user.password],
        )
        .map_err(|e| CustomError::DBInsertError(e))?)
}

fn get_db_url() -> Result<String, VarError> {
    dotenv().ok();
    Ok(env::var("DATABASE_URL")?)
}
