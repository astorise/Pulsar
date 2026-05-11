use crate::protocol::{ClientMessage, ServerMessage, decode_server_message, encode_client_message};
use anyhow::{Context, bail};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub async fn run(
    endpoint: String,
    init: ClientMessage,
    mut rx: mpsc::Receiver<ClientMessage>,
) -> anyhow::Result<()> {
    let (socket, _) = connect_async(&endpoint)
        .await
        .with_context(|| format!("failed to connect to orchestrator at {endpoint}"))?;
    let (mut writer, mut reader) = socket.split();

    writer
        .send(Message::Text(encode_client_message(&init)?.into()))
        .await
        .context("failed to send init message")?;

    loop {
        tokio::select! {
            Some(message) = rx.recv() => {
                writer
                    .send(Message::Text(encode_client_message(&message)?.into()))
                    .await
                    .context("failed to send user message")?;
            }
            next = reader.next() => {
                let Some(message) = next else {
                    break;
                };
                handle_ws_message(message?)?;
            }
        }
    }

    Ok(())
}

pub fn handle_ws_text(payload: &str) -> anyhow::Result<String> {
    match decode_server_message(payload)? {
        ServerMessage::StreamToken { content } => Ok(content),
        ServerMessage::ActionEvent { action, target } => Ok(format!("[Agent {action}: {target}]")),
        ServerMessage::Error { message } => Ok(format!("[Agent error: {message}]")),
    }
}

fn handle_ws_message(message: Message) -> anyhow::Result<()> {
    match message {
        Message::Text(payload) => {
            print!("{}", handle_ws_text(&payload)?);
            Ok(())
        }
        Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => Ok(()),
        Message::Close(_) => bail!("orchestrator closed the websocket"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_token_is_printable_content() {
        let text = handle_ws_text(r#"{"type":"stream_token","content":"abc"}"#).unwrap();

        assert_eq!(text, "abc");
    }

    #[test]
    fn action_event_is_system_log() {
        let text = handle_ws_text(
            r#"{"type":"action_event","action":"write_file","target":"src/lib.rs"}"#,
        )
        .unwrap();

        assert_eq!(text, "[Agent write_file: src/lib.rs]");
    }
}
