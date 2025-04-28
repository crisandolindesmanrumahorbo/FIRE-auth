use request_http_parser::parser::Request;

use crate::{
    constants::{BAD_REQUEST, INTERNAL_ERROR, NO_CONTENT, OK_RESPONSE, UNAUTHORIZED},
    error::CustomError,
    utils::{
        create_jwt, des_from_str, encrypt, extract_token, is_password_valid, ser_to_str, verify_jwt,
    },
};

use super::repo::{AuthRepository, DbConnection};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Response {
    pub token: String,
}

pub struct AuthService<DB>
where
    DB: DbConnection + Send + Sync + 'static,
{
    repository: AuthRepository<DB>,
}

impl<DB> AuthService<DB>
where
    DB: DbConnection + Send + Sync + 'static,
{
    pub fn new(pool: DB) -> Self {
        AuthService {
            repository: AuthRepository::new(pool),
        }
    }

    pub async fn login(&self, request: &Request) -> (String, String) {
        self.repository.print_pool_stats();

        let req_user = match &request.body {
            Some(body) => match des_from_str(body) {
                Ok(user) => user,
                Err(_) => return (UNAUTHORIZED.to_string(), "".to_string()),
            },
            None => return (UNAUTHORIZED.to_string(), "".to_string()),
        };
        let user_db = match self.repository.query_user(&req_user).await {
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

        if !is_password_valid(&req_user.password, &user_db.password) {
            println!("User {} wrong password", req_user.username);
            return (
                UNAUTHORIZED.to_string(),
                "Username or password is incorrect".to_string(),
            );
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

    pub async fn register(&self, request: &Request) -> (String, String) {
        let req_user: super::model::User = match &request.body {
            Some(body) => match des_from_str(body) {
                Ok(user) => user,
                Err(_) => return (UNAUTHORIZED.to_string(), "".to_string()),
            },
            None => return (UNAUTHORIZED.to_string(), "".to_string()),
        };

        let new_user = super::model::User {
            username: req_user.username,
            password: encrypt(&req_user.password),
            id: None,
        };
        match self.repository.insert_user(&new_user).await {
            Ok(_) => (NO_CONTENT.to_string(), "".to_string()),
            Err(err) => match err {
                CustomError::UsernameExists => {
                    eprintln!("Error insert: {:#?}", err);
                    (BAD_REQUEST.to_string(), "Already registered".to_string())
                }
                error => {
                    eprintln!("Error insert user db: {:#?}", error);
                    (INTERNAL_ERROR.to_string(), "".to_string())
                }
            },
        }
    }

    pub fn validate(&self, request: &Request) -> (String, String) {
        let token = match extract_token(&request.headers) {
            Some(token) => token,
            None => {
                println!("Missing Header");
                return (UNAUTHORIZED.to_string(), "".to_string());
            }
        };

        match verify_jwt(&token) {
            Ok(_) => (OK_RESPONSE.to_string(), "".to_string()),
            Err(err) => {
                println!("Verification failed: {}", err);
                (UNAUTHORIZED.to_string(), "".to_string())
            }
        }
    }
}
