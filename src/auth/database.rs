use bcrypt::verify;
use std::env::{self, VarError};

use crate::auth::model::User;
use dotenvy::dotenv;
use postgres::{Client, NoTls, error::SqlState};

pub enum AuthDatabaseError {
    CONNECTION,
    NULL,
    DuplicateKey,
}

pub fn check_user_db(user_login: User) -> Result<User, AuthDatabaseError> {
    let url = match get_db_url() {
        Ok(url) => url,
        Err(e) => {
            print!("Missing DATABASE_URL {}", e);
            return Err(AuthDatabaseError::CONNECTION);
        }
    };
    let mut client = match Client::connect(&url, NoTls) {
        Ok(client) => client,
        Err(_) => return Err(AuthDatabaseError::CONNECTION),
    };
    let rows = match client.query(
        "SELECT * FROM users WHERE username = $1",
        &[&user_login.username],
    ) {
        Ok(rows) => rows,
        Err(e) => {
            if let Some(code) = e.code() {
                match code {
                    &SqlState::SYNTAX_ERROR => {
                        println!("Syntax error in SQL query.");
                        return Err(AuthDatabaseError::NULL);
                    }
                    _ => {
                        println!("Other database error: {}", e);
                        return Err(AuthDatabaseError::NULL);
                    }
                }
            } else {
                println!("Unknown database error: {}", e);
                return Err(AuthDatabaseError::NULL);
            }
        }
    };
    if rows.len() > 1 {
        println!("Multiple user found");
        return Err(AuthDatabaseError::NULL);
    };
    if rows.len() == 0 {
        println!("No entry found.");
        return Err(AuthDatabaseError::NULL);
    };
    let user = match rows.get(0) {
        Some(row) => User {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
        },
        None => {
            println!("multiple user");
            return Err(AuthDatabaseError::NULL);
        }
    };
    if verify(user_login.password, &user.password).unwrap_or(false) {
        Ok(user)
    } else {
        println!("user {} wrong password", user_login.username);
        Err(AuthDatabaseError::NULL)
    }
}

pub fn insert_db_user(new_user: User) -> Result<u64, AuthDatabaseError> {
    let url = match get_db_url() {
        Ok(url) => url,
        Err(e) => {
            print!("Missing DATABASE_URL {}", e);
            return Err(AuthDatabaseError::CONNECTION);
        }
    };
    let mut client = match Client::connect(&url, NoTls) {
        Ok(client) => client,
        Err(_) => return Err(AuthDatabaseError::CONNECTION),
    };
    match client.execute(
        "INSERT INTO users (username, password) VALUES ($1, $2)",
        &[&new_user.username, &new_user.password],
    ) {
        Ok(id) => {
            print!("User created {}", id);
            return Ok(id);
        }
        Err(e) => {
            if let Some(code) = e.code() {
                match code {
                    &SqlState::UNIQUE_VIOLATION => {
                        println!("Duplicate entry found.");
                        return Err(AuthDatabaseError::DuplicateKey);
                    }
                    &SqlState::SYNTAX_ERROR => {
                        println!("Syntax error in SQL query.");
                        return Err(AuthDatabaseError::NULL);
                    }
                    _ => {
                        println!("Other database error: {}", e);
                        return Err(AuthDatabaseError::NULL);
                    }
                }
            } else {
                println!("Unknown database error: {}", e);
                return Err(AuthDatabaseError::NULL);
            }
        }
    }
}

fn get_db_url() -> Result<String, VarError> {
    dotenv().ok();
    Ok(env::var("DATABASE_URL")?)
}
