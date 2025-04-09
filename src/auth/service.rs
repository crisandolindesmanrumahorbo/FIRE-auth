use super::repository::AuthRepository;
use crate::{
    constants::{BAD_REQUEST, INTERNAL_ERROR, NO_CONTENT, OK_RESPONSE, UNAUTHORIZED},
    error::CustomError,
    req::Request,
    utils::{
        create_jwt, des_from_str, encrypt, extract_token, is_password_valid, ser_to_str, verify_jwt,
    },
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Response {
    pub token: String,
}

pub struct AuthService {
    respository: AuthRepository,
}

impl AuthService {
    pub fn new(pool: sqlx::AnyPool) -> Self {
        AuthService {
            respository: AuthRepository::new(pool),
        }
    }

    pub async fn login(&self, request: &str) -> (String, String) {
        self.respository.print_pool_stats();
        let req_user = match des_from_str(request) {
            Ok(user) => user,
            Err(_) => return (UNAUTHORIZED.to_string(), "".to_string()),
        };
        let user_db = match self.respository.query_user(&req_user).await {
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

    pub async fn register(&self, request: &str) -> (String, String) {
        let req_user: super::model::User = match des_from_str(request) {
            Ok(user) => user,
            Err(_) => return (BAD_REQUEST.to_string(), "invalid body".to_string()),
        };

        let new_user = super::model::User {
            username: req_user.username,
            password: encrypt(&req_user.password),
            id: None,
        };
        match self.respository.insert_user(&new_user).await {
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
