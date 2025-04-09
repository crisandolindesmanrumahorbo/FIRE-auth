use crate::auth::service::AuthService;
use crate::constants::NOT_FOUND;
use crate::req::Method::{GET, POST};
use crate::req::Request;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot::Receiver;

pub struct Server {
    auth_svc: Arc<AuthService>,
}

impl Server {
    pub fn new(auth_svc: Arc<AuthService>) -> Self {
        Self { auth_svc }
    }

    pub async fn start(&self, mut shutdown_rx: Receiver<()>) -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:7879")
            .await
            .expect("failed to binding port");
        println!("Server running on http://127.0.0.1:7879");

        loop {
            tokio::select! {
                conn = listener.accept() => {
                    let (mut stream, _) = conn?;

                    let auth_svc = Arc::clone(&self.auth_svc);

                    tokio::spawn(async move {
                    let (reader, writer) = stream.split();
                        if let Err(e) = Self::handle_client(reader, writer, &auth_svc).await {
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

    pub async fn handle_client<Reader, Writer>(
        reader: Reader,
        mut writer: Writer,
        auth_svc: &Arc<AuthService>,
    ) -> Result<()>
    where
        Reader: AsyncRead + Unpin,
        Writer: AsyncWrite + Unpin,
    {
        let request = Request::new(reader)
            .await
            .context("Failed to read request")?;

        // Route
        let (status_line, content) = match (&request.method, request.path.as_str()) {
            (POST, "/login") => auth_svc.login(&request.body).await,
            (POST, "/register") => auth_svc.register(&request.body).await,
            (GET, "/validate") => auth_svc.validate(&request),
            _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
        };

        writer
            .write_all(format!("{}{}", status_line, content).as_bytes())
            .await
            .context("Failed to write")
    }
}
