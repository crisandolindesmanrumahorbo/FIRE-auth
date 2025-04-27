use crate::common::insert_db_user;
use common::setup_test_db;
use std::collections::HashMap;
use stockbit_auth::{
    auth::{model::User, service::AuthService},
    constants::{OK_RESPONSE, UNAUTHORIZED},
    req::Request,
    utils::{encrypt, ser_to_str},
};
mod common;

#[cfg(test)]
pub async fn get_db_pool() -> sqlx::SqlitePool {
    let pool = setup_test_db().await;

    // Check if table exists before inserting
    let check = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users';")
        .fetch_one(&pool)
        .await;
    assert!(check.is_ok(), "❌ The 'users' table does not exist!");
    pool
}

#[cfg(feature = "test-sqlite")]
#[tokio::test]
async fn login_user_success() {
    let pool = get_db_pool().await;
    let auth_user = User {
        username: "test_1".to_string(),
        password: "hashed_password".to_string(),
        id: None,
    };
    insert_db_user(&auth_user.username, &encrypt(&auth_user.password), &pool).await;
    let body = Some(ser_to_str(&auth_user).expect("failed to serialized"));
    let service = AuthService::new(pool);

    let response = service
        .login(&Request {
            body,
            method: stockbit_auth::req::Method::POST,
            path: "/login".to_string(),
            params: None,
            headers: HashMap::new(),
        })
        .await;

    assert_eq!(response.0, OK_RESPONSE.to_string());
}

#[cfg(feature = "test-sqlite")]
#[tokio::test]
async fn login_user_unauthorized_not_registered() {
    let pool = get_db_pool().await;
    let non_auth_user = User {
        username: "test_2".to_string(),
        password: "hashed_password".to_string(),
        id: None,
    };
    let body = Some(ser_to_str(&non_auth_user).expect("failed to serialized"));
    let controller = AuthService::new(pool);

    let response = controller
        .login(&Request {
            body,
            method: stockbit_auth::req::Method::POST,
            path: "/login".to_string(),
            params: None,
            headers: HashMap::new(),
        })
        .await;

    assert_eq!(response.0, UNAUTHORIZED.to_string());
}

#[cfg(feature = "test-sqlite")]
#[tokio::test]
async fn login_user_unauthorized_wrong_password() {
    let pool = get_db_pool().await;
    let auth_user = User {
        username: "test_3".to_string(),
        password: "hashed_password".to_string(),
        id: None,
    };
    insert_db_user(&auth_user.username, &encrypt("different password"), &pool).await;
    let body = Some(ser_to_str(&auth_user).expect("failed to serialized"));
    let controller = AuthService::new(pool);

    let response = controller
        .login(&Request {
            body,
            method: stockbit_auth::req::Method::POST,
            path: "/login".to_string(),
            params: None,
            headers: HashMap::new(),
        })
        .await;

    assert_eq!(response.0, UNAUTHORIZED.to_string());
    assert_eq!(response.1, "Username or password is incorrect".to_string());
}

#[cfg(feature = "test-sqlite")]
#[tokio::test]
async fn handle_client_user_success() {
    let pool = get_db_pool().await;
    let username = "crisandolin";
    let password = "rumbo";
    let auth_user = User {
        username: username.to_string(),
        password: password.to_string(),
        id: None,
    };
    insert_db_user(&auth_user.username, &encrypt(&auth_user.password), &pool).await;
    let svc = AuthService::new(pool.clone());
    let controller = std::sync::Arc::new(svc);
    let reader = tokio_test::io::Builder::new()
        .read(
            format!(
                "POST /login HTTP/1.1\r\n\
                Content-Type: application/json\r\n\
                User-Agent: Test\r\n\
                Content-Length: {}\r\n\
                \r\n\
                {{\"username\": \"{}\",\"password\": \"{}\"}}",
                44, username, password
            )
            .as_bytes(),
        )
        .build();

    // Create a simple in-memory writer
    let mut output = Vec::new();
    let writer = common::TestWriter(&mut output);

    let _ = stockbit_auth::server::Server::handle_client(reader, writer, &controller).await;

    let result = String::from_utf8_lossy(&output);
    println!("Server wrote: {result}");
    assert!(result.contains("200 OK"));
    assert!(result.contains("token"));
}
