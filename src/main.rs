mod api;
mod error;
mod imap;

use anyhow::{Context, Result};

async fn handle_connection(mut client: imap::Client) -> Result<()> {
    client.greet().await.context("Failed to greet client")?;

    loop {
        let message = client
            .wait_message()
            .await
            .context("Failed to wait client message")?;

        let result = match message {
            imap::Message::Capability { id } => client.send_capability(&id).await,
            imap::Message::Login {
                id,
                email,
                password,
            } => {
                let Some((firstname, lastname)) =
                    email.split_once('@').and_then(|(l, _)| l.split_once('.'))
                else {
                    return client.bad_credentials(&id).await;
                };
                let username = format!(
                    "e-{}{}",
                    firstname.chars().next().unwrap_or_default(),
                    lastname.chars().take(7).collect::<String>()
                );
                match client.api.login(&username, &password).await {
                    Ok(Some(token)) => client.login(&id, email, token).await,
                    Ok(None) => client.bad_credentials(&id).await,
                    Err(err) => {
                        error::print(&err);
                        client.internal_error(&id, &err).await
                    }
                }
            }
            imap::Message::NoOp { id } => client.noop(&id).await,
            imap::Message::End => Ok(()),
            imap::Message::Unknown { id, command, args } => {
                let args = args
                    .into_iter()
                    .map(imap::Data::into_string)
                    .collect::<Vec<_>>();
                println!("{} {} {}", id, command, args.join(" "));
                client.unknown_command(&id, &command).await
            }
            imap::Message::Bad(message) => {
                println!("{}", message);
                client.bad_request().await
            }
        };
        if let Err(err) = result {
            error::print(&err);
        }
    }
}

async fn run() -> Result<()> {
    let server = imap::Server::new()
        .await
        .context("Failed to initialize imap server")?;
    loop {
        let Ok(client) = server
            .wait_client()
            .await
            .context("Failed to wait connection")
            .map_err(|err| error::print(&err))
        else {
            continue;
        };

        tokio::spawn(async move {
            if let Err(err) = handle_connection(client).await {
                error::print(&err);
            }
            println!("------");
        });
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        error::print(&err);
    }
}
