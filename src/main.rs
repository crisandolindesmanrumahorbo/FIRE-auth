use anyhow::{Context, Result};
use stockbit_auth::auth::{self, login, register};
use stockbit_auth::config;
use stockbit_auth::constants::NOT_FOUND;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // init config
    config::init_config();

    // init DB_POOL
    let db_pool = auth::database::get_db_pool().await;

    let listener = TcpListener::bind("127.0.0.1:7879")
        .await
        .expect("Failed to bind to port");
    println!("Server running on http://127.0.0.1:7879");

    loop {
        auth::database::print_pool_stats(db_pool).await;
        let (stream, _) = listener.accept().await.context("failed to accept")?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    let size = stream
        .read(&mut buffer)
        .await
        .context("Failed to read stream")?;
    let request = String::from_utf8_lossy(&buffer[..size]);

    let (status_line, content) = match &*request {
        r if r.starts_with("POST /login") => login(r).await,
        r if r.starts_with("POST /register") => register(r).await,
        _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
    };

    stream
        .write_all(format!("{}{}", status_line, content).as_bytes())
        .await
        .context("Failed to write")
}
