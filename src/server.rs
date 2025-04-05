use crate::{auth::controller::AuthController, constants::NOT_FOUND};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    auth_controller: Arc<AuthController>,
}

impl Server {
    pub fn new(auth_controller: Arc<AuthController>) -> Self {
        Self { auth_controller }
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:7879")
            .await
            .expect("Failed to bind to port");
        println!("Server running on http://127.0.0.1:7879");
        loop {
            let (stream, _) = listener.accept().await.context("failed to accept")?;
            let controller = Arc::clone(&self.auth_controller);
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(stream, &controller).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }

    async fn handle_client(
        mut stream: TcpStream,
        auth_controller: &Arc<AuthController>,
    ) -> Result<()> {
        let mut buffer = [0; 1024];
        let size = stream
            .read(&mut buffer)
            .await
            .context("Failed to read stream")?;
        let request = String::from_utf8_lossy(&buffer[..size]);

        let (status_line, content) = match &*request {
            r if r.starts_with("POST /login") => auth_controller.login(r).await,
            r if r.starts_with("POST /register") => auth_controller.register(r).await,
            _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
        };

        stream
            .write_all(format!("{}{}", status_line, content).as_bytes())
            .await
            .context("Failed to write")
    }
}
