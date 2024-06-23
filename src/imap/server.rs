use super::Client;
use anyhow::{Context as _, Result};
use tokio::io::BufStream;
use tokio::net::TcpListener;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("0.0.0.0:5000")
            .await
            .context("Failed to bind TCP address")?;
        Ok(Self { listener })
    }

    pub async fn wait_client(&self) -> Result<Client> {
        self.listener
            .accept()
            .await
            .map(|(stream, _)| Client::new(BufStream::new(stream)))
            .context("Failed to accept TCP connection")
    }
}
