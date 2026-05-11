mod git;
mod protocol;
mod repl;
mod webdav;
mod ws_client;

use anyhow::Context;
use std::{
    env,
    io::{self, Write},
    net::SocketAddr,
    path::PathBuf,
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = CliConfig::from_env()?;
    let token = webdav::generate_token();
    let sandbox = git::Sandbox::create(&config.workspace_root, &token)?;
    let webdav_root = sandbox
        .as_ref()
        .map(|sandbox| sandbox.worktree_path.clone())
        .unwrap_or_else(|| config.workspace_root.clone());
    let (webdav_url, webdav_task) =
        webdav::spawn(webdav_root, config.webdav_addr, token.clone()).await?;

    let init = protocol::ClientMessage::Init {
        workspace_url: webdav_url,
        workspace_token: token,
    };
    let (tx, rx) = mpsc::channel(64);
    let (finish_tx, mut finish_rx) = mpsc::channel(1);
    let ws_task = tokio::spawn(ws_client::run(config.orchestrator_url, init, rx, finish_tx));
    let repl_task = tokio::spawn(repl::run(tx));

    tokio::select! {
        _ = finish_rx.recv() => {
            if let Some(sandbox) = sandbox.as_ref() {
                handle_sandbox_finish(sandbox)?;
            }
        }
        result = ws_task => result.context("websocket task join failed")??,
        result = repl_task => result.context("repl task join failed")??,
        result = webdav_task => result.context("webdav task join failed")??,
    }

    Ok(())
}

fn handle_sandbox_finish(sandbox: &git::Sandbox) -> anyhow::Result<()> {
    println!("\nPulsar session finished. Sandbox diff:");
    let diff = sandbox.diff_stat()?;
    if diff.trim().is_empty() {
        println!("No changes to apply.");
        sandbox.cleanup()?;
        return Ok(());
    }

    println!("{diff}");
    if prompt_yes_no("Apply these changes? [y/N] ")? {
        sandbox.apply_patch_to_repo()?;
        println!("Changes applied to the active worktree index.");
    } else {
        println!("Changes discarded.");
    }
    sandbox.cleanup()?;
    Ok(())
}

fn prompt_yes_no(prompt: &str) -> anyhow::Result<bool> {
    print!("{prompt}");
    io::stdout().flush().context("failed to flush stdout")?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("failed to read user confirmation")?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes" | "YES"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliConfig {
    orchestrator_url: String,
    webdav_addr: SocketAddr,
    workspace_root: PathBuf,
}

impl CliConfig {
    fn from_env() -> anyhow::Result<Self> {
        let orchestrator_url = env::var("PULSAR_ORCHESTRATOR_WS")
            .unwrap_or_else(|_| "ws://127.0.0.1:8081/orchestrator".to_string());
        let webdav_addr = env::var("PULSAR_WEBDAV_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:0".to_string())
            .parse()
            .context("PULSAR_WEBDAV_ADDR must be a socket address")?;
        let workspace_root = env::current_dir().context("failed to resolve current directory")?;

        Ok(Self {
            orchestrator_url,
            webdav_addr,
            workspace_root,
        })
    }
}
