use common::setup_test_db;
use stockbit_auth::{
    auth::{
        controller::AuthController, model::User, repository::AuthRepository, service::AuthService,
    },
    constants::{OK_RESPONSE, UNAUTHORIZED},
    utils::{encrypt, ser_to_str},
};
mod common;

#[cfg(test)]
pub async fn get_db_pool() -> sqlx::AnyPool {
    let pool = setup_test_db().await;
    pool
}

#[tokio::test]
async fn login_user_success() {
    let pool = get_db_pool().await;

    // Check if table exists before inserting
    let check = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users';")
        .fetch_one(&pool)
        .await;
    assert!(check.is_ok(), "❌ The 'users' table does not exist!");
    let auth_user = User {
        username: "test_1".to_string(),
        password: "hashed_password".to_string(),
        id: None,
    };
    sqlx::query("INSERT INTO users (username, password) VALUES (?, ?);")
        .bind(&auth_user.username)
        .bind(encrypt(&auth_user.password))
        .execute(&pool)
        .await
        .expect("Failed to insert test user");
    let repository = AuthRepository::new(pool).await.expect("Failed create repo");
    let svc = AuthService::new(repository);
    let controller = AuthController::new(svc);

    let response = controller.login(&ser_to_str(&auth_user).unwrap()).await;

    assert_eq!(response.0, OK_RESPONSE.to_string());
}

#[tokio::test]
async fn login_user_unauthorized_not_registered() {
    let pool = get_db_pool().await;

    // Check if table exists before inserting
    let check = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users';")
        .fetch_one(&pool)
        .await;
    assert!(check.is_ok(), "❌ The 'users' table does not exist!");
    let non_auth_user = User {
        username: "test_2".to_string(),
        password: "hashed_password".to_string(),
        id: None,
    };

    let repository = AuthRepository::new(pool).await.expect("Failed create repo");
    let svc = AuthService::new(repository);
    let controller = AuthController::new(svc);
    let response = controller.login(&ser_to_str(&non_auth_user).unwrap()).await;

    assert_eq!(response.0, UNAUTHORIZED.to_string());
}

#[tokio::test]
async fn login_user_unauthorized_wrong_password() {
    let pool = get_db_pool().await;

    // Check if table exists before inserting
    let check = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users';")
        .fetch_one(&pool)
        .await;
    assert!(check.is_ok(), "❌ The 'users' table does not exist!");
    let auth_user = User {
        username: "test_3".to_string(),
        password: "hashed_password".to_string(),
        id: None,
    };

    sqlx::query("INSERT INTO users (username, password) VALUES (?, ?);")
        .bind(&auth_user.username)
        .bind(encrypt("different password"))
        .execute(&pool)
        .await
        .expect("Failed to insert test user");

    let repository = AuthRepository::new(pool).await.expect("Failed create repo");
    let svc = AuthService::new(repository);
    let controller = AuthController::new(svc);
    let response = controller.login(&ser_to_str(&auth_user).unwrap()).await;

    assert_eq!(response.0, UNAUTHORIZED.to_string());
    assert_eq!(response.1, "Username or password is incorrect".to_string());
}
