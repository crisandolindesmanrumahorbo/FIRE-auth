use crate::auth::repo::DbConnection;
use crate::auth::service::AuthService;
use crate::constants::{BAD_REQUEST, NOT_FOUND};
use anyhow::{Context, Result};
use request_http_parser::parser::{Method, Request};

use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot::Receiver;

pub struct Server<DB>
where
    DB: DbConnection + Send + Sync + 'static,
{
    auth_svc: Arc<AuthService<DB>>,
}

impl<DB> Server<DB>
where
    DB: DbConnection + Send + Sync + 'static,
{
    pub fn new(pool: DB) -> Self {
        let auth_svc = Arc::new(AuthService::new(pool));

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
                        if let Err(e) = Server::handle_client(reader, writer, &auth_svc).await {
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
        mut reader: Reader,
        mut writer: Writer,
        auth_svc: &Arc<AuthService<DB>>,
    ) -> Result<()>
    where
        Reader: AsyncRead + Unpin,
        Writer: AsyncWrite + Unpin,
    {
        let mut buffer = [0; 1024];
        let size = reader
            .read(&mut buffer)
            .await
            .context("Failed to read stream")?;
        if size >= 1024 {
            let _ = writer
                .write_all(format!("{}{}", BAD_REQUEST, "Requets too large").as_bytes())
                .await
                .context("Failed to write");

            let _ = writer.flush().await.context("Failed to flush");

            return Ok(());
        }
        let request = String::from_utf8_lossy(&buffer[..size]);
        let request = match Request::new(&request) {
            Ok(req) => req,
            Err(e) => {
                println!("{}", e);
                let _ = writer
                    .write_all(format!("{}{}", BAD_REQUEST, e).as_bytes())
                    .await
                    .context("Failed to write");

                let _ = writer.flush().await.context("Failed to flush");
                return Ok(());
            }
        };

        // Route
        let (status_line, content) = match (&request.method, request.path.as_str()) {
            (Method::POST, "/login") => auth_svc.login(&request).await,
            (Method::POST, "/register") => auth_svc.register(&request).await,
            (Method::GET, "/validate") => auth_svc.validate(&request),
            _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
        };

        writer
            .write_all(format!("{}{}", status_line, content).as_bytes())
            .await
            .context("Failed to write")
    }
}
