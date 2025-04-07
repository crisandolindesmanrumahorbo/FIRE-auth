use std::sync::Arc;

use stockbit_auth::auth::controller::AuthController;
use stockbit_auth::auth::repository::AuthRepository;
use stockbit_auth::auth::service::AuthService;
use stockbit_auth::config::{self};
use stockbit_auth::database::Database;
use stockbit_auth::server::Server;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // init config
    config::init_config();

    // init DB
    let db_pool = Database::new_pool(&config::get_config().database_url).await;

    let auth_repo = AuthRepository::new(db_pool).await.expect("Error init repo");
    let auth_svc = AuthService::new(auth_repo);
    let auth_controller = Arc::new(AuthController::new(auth_svc));
    let server = Server::new(auth_controller);
    server.start().await.expect("Failed start server");
}
