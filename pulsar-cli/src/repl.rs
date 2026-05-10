use anyhow::Result;
use rustyline::{error::ReadlineError, DefaultEditor};
use tracing::debug;

/// Interactive REPL that reads user tasks and forwards them to the orchestrator.
///
/// `readline` is inherently blocking. In this scaffold the REPL is called via
/// `tokio::task::block_in_place` in `main.rs` so it does not starve the async
/// runtime. When the async orchestrator is fully wired in, each input line will
/// be dispatched as a `tokio::spawn` and the REPL will await the response.
pub struct Repl {
    editor: DefaultEditor,
}

impl Repl {
    pub fn new() -> Result<Self> {
        Ok(Self {
            editor: DefaultEditor::new()?,
        })
    }

    /// Block until the user types `/exit` or sends EOF / Ctrl-C.
    pub fn run(&mut self) -> Result<()> {
        println!("Pulsar ready. Type your task or /exit to quit.");
        loop {
            match self.editor.readline("pulsar> ") {
                Ok(line) => {
                    let input = line.trim().to_string();
                    if input.is_empty() {
                        continue;
                    }
                    let _ = self.editor.add_history_entry(&input);

                    if input == "/exit" || input == "/quit" {
                        break;
                    }

                    debug!(input, "dispatching user input");
                    // TODO(orchestrator): route to Orchestrator::handle(input).await
                    println!("[stub] task queued: {input}");
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                Err(e) => return Err(e.into()),
            }
        }
        println!("Goodbye.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repl_constructs_without_error() {
        // Smoke test: DefaultEditor initialises correctly in a headless env.
        assert!(Repl::new().is_ok());
    }
}
