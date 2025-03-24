use bcrypt::verify;
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::errors::Error;
use jsonwebtoken::{EncodingKey, Header, encode};
use postgres::{Client, NoTls};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{env, thread};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    pub id: Option<i32>,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

enum AuthDatabaseError {
    CONNECTION,
    QUERY,
    NULL,
}

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const UNATHORIZED: &str = "HTTP/1.1 401 Unathorized\r\n\r\n";
const INTERNAL_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7879").unwrap();
    println!("Server running on http://127.0.0.1:7879");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(err) => println!("unable to connect: {}", err),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if r.starts_with("POST /login") => handle_post_request(r),
                _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
            };

            stream
                .write_all(format!("{}{}", status_line, content).as_bytes())
                .unwrap();
        }
        Err(e) => eprintln!("Unable to read stream: {}", e),
    }
}

fn create_jwt(user: User) -> Result<String, Error> {
    let private_key = get_private_key();
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24)) // Token valid for 24 hours
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.username,
        exp: expiration,
    };

    encode(
        &Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &private_key,
    )
}

fn check_db(user_login: User) -> Result<User, AuthDatabaseError> {
    let mut client = match Client::connect(&get_db_url(), NoTls) {
        Ok(client) => client,
        Err(_) => return Err(AuthDatabaseError::CONNECTION),
    };
    let row = match client.query_one(
        "SELECT * FROM users WHERE username = $1",
        &[&user_login.username],
    ) {
        Ok(row) => row,
        Err(_) => return Err(AuthDatabaseError::QUERY),
    };
    let user = User {
        id: row.get(0),
        username: row.get(1),
        password: row.get(2),
    };
    if verify(user_login.password, &user.password).unwrap_or(false) {
        Ok(user)
    } else {
        Err(AuthDatabaseError::NULL)
    }
}

fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}

fn handle_post_request(request: &str) -> (String, String) {
    let req_user = match get_user_request_body(&request) {
        Ok(user) => user,
        Err(_) => return (NOT_FOUND.to_string(), "body not valid".to_string()),
    };
    let user_db = match check_db(req_user) {
        Ok(user) => user,
        Err(why) => match why {
            AuthDatabaseError::CONNECTION => {
                println!("Failed connect to db");
                return (INTERNAL_ERROR.to_string(), "".to_string());
            }
            AuthDatabaseError::QUERY => {
                println!("Failed query to db");
                return (INTERNAL_ERROR.to_string(), "".to_string());
            }
            AuthDatabaseError::NULL => {
                println!("User not found");
                return (UNATHORIZED.to_string(), "Unauthorized".to_string());
            }
        },
    };
    let token = match create_jwt(user_db) {
        Ok(token) => token,
        Err(_) => {
            println!("jwt error");
            return (INTERNAL_ERROR.to_string(), "".to_string());
        }
    };
    let response = Response { token };
    let response_json = match serde_json::to_string(&response) {
        Ok(json) => json,
        Err(_) => {
            println!("serde error");
            return (INTERNAL_ERROR.to_string(), "".to_string());
        }
    };
    (OK_RESPONSE.to_string(), response_json)
}

fn get_private_key() -> EncodingKey {
    dotenv().ok();
    let key = env::var("JWT_PRIVATE_KEY").expect("Missing JWT_PRIVATE_KEY");
    EncodingKey::from_rsa_pem(key.replace("\\n", "\n").as_bytes()).expect("Invalid private key")
}

fn get_db_url() -> String {
    dotenv().ok();
    let key = env::var("DATABASE_URL").expect("Missing DATABASE_URL");
    key
}
