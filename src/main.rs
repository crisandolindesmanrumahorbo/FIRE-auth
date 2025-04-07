use std::sync::Arc;

use sqlx::Pool;
use stockbit_auth::auth::controller::AuthController;
use stockbit_auth::config::{self};
use stockbit_auth::database::Database;
use stockbit_auth::server::Server;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::oneshot;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    // Init config and DB
    config::init_config();
    let db_pool = Database::new_pool(&config::get_config().database_url).await;
    let auth_controller = Arc::new(AuthController::new(db_pool.clone()));

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Start server
    let server = Server::new(auth_controller);
    let server_handle = tokio::spawn(async move { server.start(shutdown_rx).await });
    gracefully_shutdown(db_pool, shutdown_tx, server_handle).await;
    Ok(())
}

async fn gracefully_shutdown(
    db_pool: Pool<sqlx::Any>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    server_handle: tokio::task::JoinHandle<Result<(), anyhow::Error>>,
) {
    // Wait for shutdown signal
    let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
    let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = signal_terminate.recv() => {
            println!("Shutdown signal received");
        },
        _ = signal_interrupt.recv() => {
            println!("SIGINT received");
        }
    }

    // Trigger graceful shutdown
    let _ = shutdown_tx.send(());
    let _ = server_handle.await;

    // Close DB pool
    db_pool.close().await;

    println!("Shutdown completed");
}
