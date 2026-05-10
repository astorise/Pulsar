mod config;
mod mesh_client;
mod orchestrator;
mod repl;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Pulsar starting");

    let cfg = config::Config::load()?;
    info!(endpoint = %cfg.backend.tachyon_endpoint, "config loaded");

    // Lazy connect: defers TCP handshake to first RPC call.
    // Pulsar stays usable even when the Tachyon node is temporarily unreachable.
    let _client = mesh_client::TachyonMeshClient::new_lazy(&cfg.backend.tachyon_endpoint)?;
    info!(endpoint = %cfg.backend.tachyon_endpoint, "TachyonMeshClient ready");

    let mut repl = repl::Repl::new()?;
    // block_in_place: informs tokio that readline will block, so it can
    // schedule other tasks on remaining threads during the wait.
    tokio::task::block_in_place(|| repl.run())?;

    Ok(())
}
