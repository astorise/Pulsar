use crate::protocol::ClientMessage;
use anyhow::Context;
use rustyline::DefaultEditor;
use tokio::sync::mpsc;

pub async fn run(tx: mpsc::Sender<ClientMessage>) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || run_blocking(tx))
        .await
        .context("repl thread panicked")?
}

fn run_blocking(tx: mpsc::Sender<ClientMessage>) -> anyhow::Result<()> {
    let mut editor = DefaultEditor::new().context("failed to initialize terminal editor")?;

    loop {
        match editor.readline("pulsar> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                    break;
                }
                if trimmed.is_empty() {
                    continue;
                }

                let _ = editor.add_history_entry(trimmed);
                tx.blocking_send(ClientMessage::UserMessage {
                    content: trimmed.to_string(),
                })
                .context("websocket sender closed")?;
            }
            Err(rustyline::error::ReadlineError::Interrupted)
            | Err(rustyline::error::ReadlineError::Eof) => break,
            Err(err) => return Err(err).context("failed to read terminal input"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::protocol::ClientMessage;

    #[test]
    fn user_message_variant_is_available_for_repl() {
        let msg = ClientMessage::UserMessage {
            content: "run tests".to_string(),
        };

        assert!(matches!(msg, ClientMessage::UserMessage { .. }));
    }
}
