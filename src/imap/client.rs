use crate::api::ApiClient;

use super::Message;
use anyhow::{Context, Error, Result};
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufStream};
use tokio::net::TcpStream;

macro_rules! send {
    ($stream:expr, $message:expr) => {{
        let result = $stream.write_all($message).await;
        if result.is_ok() {
            $stream.flush().await
        } else {
            result
        }
    }};
}

pub struct Client {
    stream: BufStream<TcpStream>,
    email: Option<String>,
    token: Option<String>,
    pub api: ApiClient,
}

impl Client {
    pub fn new(stream: BufStream<TcpStream>) -> Self {
        Self {
            stream,
            email: None,
            token: None,
            api: ApiClient::new(),
        }
    }

    pub async fn greet(&mut self) -> Result<()> {
        send!(self.stream, b"* OK IMAP4rev1 server ready\r\n").context("Failed to greet client")
    }

    pub async fn send_capability(&mut self, id: &str) -> Result<()> {
        send!(
            self.stream,
            format!("* CAPABILITY IMAP4rev1\r\n{id} OK CAPABILITY completed\r\n").as_bytes()
        )
        .context("Failed so send capability")
    }

    pub async fn login(&mut self, id: &str, email: String, token: String) -> Result<()> {
        self.email = Some(email);
        self.token = Some(token);
        send!(
            self.stream,
            format!("{id} OK LOGIN completed\r\n").as_bytes()
        )
        .context("Failed to send login response")
    }

    pub async fn noop(&mut self, id: &str) -> Result<()> {
        send!(
            self.stream,
            format!("{id} OK NOOP completed\r\n").as_bytes()
        )
        .context("Failed to send login response")
    }

    pub async fn bad_credentials(&mut self, id: &str) -> Result<()> {
        self.token = None;
        send!(
            self.stream,
            format!("{id} NO bad credentials\r\n").as_bytes()
        )
        .context("Failed to send login response")
    }

    pub async fn bad_request(&mut self) -> Result<()> {
        send!(self.stream, b"* BAD bad request\r\n").context("Failed to send login response")
    }

    pub async fn unknown_command(&mut self, id: &str, command: &str) -> Result<()> {
        send!(
            self.stream,
            format!("{id} BAD unknown command: {command}\r\n").as_bytes()
        )
        .context("Failed to send login response")
    }

    pub async fn internal_error(&mut self, id: &str, err: &Error) -> Result<()> {
        send!(self.stream, format!("{id} BAD {err}\r\n").as_bytes())
            .context("Failed to send login response")
    }

    pub async fn wait_message(&mut self) -> Result<Message> {
        let mut message = String::new();
        self.stream.read_line(&mut message).await?;
        Ok(Message::parse(message))
    }
}
