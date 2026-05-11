mod protocol;
mod repl;
mod webdav;
mod ws_client;

use anyhow::Context;
use std::{env, net::SocketAddr, path::PathBuf};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = CliConfig::from_env()?;
    let token = webdav::generate_token();
    let (webdav_url, webdav_task) = webdav::spawn(
        config.workspace_root.clone(),
        config.webdav_addr,
        token.clone(),
    )
    .await?;

    let init = protocol::ClientMessage::Init {
        workspace_url: webdav_url,
        workspace_token: token,
    };
    let (tx, rx) = mpsc::channel(64);
    let ws_task = tokio::spawn(ws_client::run(config.orchestrator_url, init, rx));
    let repl_task = tokio::spawn(repl::run(tx));

    tokio::select! {
        result = ws_task => result.context("websocket task join failed")??,
        result = repl_task => result.context("repl task join failed")??,
        result = webdav_task => result.context("webdav task join failed")??,
    }

    Ok(())
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
