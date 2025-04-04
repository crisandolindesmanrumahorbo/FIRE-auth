pub mod database;
pub mod model;
use crate::{
    constants::{BAD_REQUEST, INTERNAL_ERROR, NOT_FOUND, NO_CONTENT, OK_RESPONSE, UNAUTHORIZED},
    error::CustomError,
    utils::{compare, create_jwt, des_from_str, encrypt, ser_to_str},
};
use model::{Response, User};

pub async fn login(request: &str) -> (String, String) {
    let req_user = match des_from_str(request) {
        Ok(user) => user,
        Err(_) => return (NOT_FOUND.to_string(), "body not valid".to_string()),
    };
    let user_db = match database::query_user(&req_user).await {
        Ok(user) => user,
        Err(why) => match why {
            CustomError::UserNotFound => {
                println!("User {} not found", req_user.username);
                return (UNAUTHORIZED.to_string(), "".to_string());
            }
            error => {
                eprintln!("Error user db: {:#?}", error);
                return (INTERNAL_ERROR.to_string(), "".to_string());
            }
        },
    };

    if !compare(&req_user.password, &user_db.password) {
        println!("User {} wrong password", req_user.username);
        return (UNAUTHORIZED.to_string(), "Username or password is incorrect".to_string());
    }

    let token = match create_jwt(user_db) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error creating JWT: {:#?}", e);
            return (INTERNAL_ERROR.to_string(), "".to_string());
        }
    };
    let response = Response { token };
    let response_json = match ser_to_str(&response) {
        Ok(json) => json,
        Err(_) => {
            println!("serde error");
            return (INTERNAL_ERROR.to_string(), "".to_string());
        }
    };
    println!("{} succeed login", req_user.username);
    (OK_RESPONSE.to_string(), response_json)
}

pub async fn register(request: &str) -> (String, String) {
    let req_user: User = match des_from_str(request) {
        Ok(user) => user,
        Err(_) => return (BAD_REQUEST.to_string(), "invalid body".to_string()),
    };

    let new_user = User {
        username: req_user.username,
        password: encrypt(&req_user.password),
        id: None,
    };
    match database::insert_user(&new_user).await {
        Ok(_) => (NO_CONTENT.to_string(), "".to_string()),
        Err(err) => match err {
            CustomError::UsernameExists => {
                eprintln!("Error insert: {:#?}", err);
                (BAD_REQUEST.to_string(), "".to_string())
            }
            error => {
                eprintln!("Error insert user db: {:#?}", error);
                (INTERNAL_ERROR.to_string(), "".to_string())
            }
        },
    }
}
