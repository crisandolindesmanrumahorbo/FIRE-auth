use crate::req::Method::{GET, POST};
use crate::req::Request;
use crate::{auth::controller::AuthController, constants::NOT_FOUND};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot::Receiver;

pub struct Server {
    auth_controller: Arc<AuthController>,
}

impl Server {
    pub fn new(auth_controller: Arc<AuthController>) -> Self {
        Self { auth_controller }
    }

    pub async fn start(&self, mut shutdown_rx: Receiver<()>) -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:7879")
            .await
            .expect("failed to binding port");
        println!("Server running on http://127.0.0.1:7879");

        loop {
            tokio::select! {
                conn = listener.accept() => {
                    let (stream, _) = conn?;
                    let controller = Arc::clone(&self.auth_controller);

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, &controller).await {
                            eprintln!("Connection error: {}", e);
                        }
                    });
                }
                // Shutdown signal check
                _ = &mut shutdown_rx => {
                    println!("Shutting down server...");
                    break;
                }
            }
        }
        Ok(())
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
        let request = Request::new(request)
            .await
            .context("Failed to read request")?;

        // Route
        let (status_line, content) = match (&request.method, request.path.as_str()) {
            (POST, "/login") => auth_controller.login(&request.body).await,
            (POST, "/register") => auth_controller.register(&request.body).await,
            (GET, "/validate") => auth_controller.validate(&request),
            _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
        };

        stream
            .write_all(format!("{}{}", status_line, content).as_bytes())
            .await
            .context("Failed to write")
    }
}
