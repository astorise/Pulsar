use anyhow::{Context, Result};
use tonic::transport::Channel;
use tracing::info;

/// gRPC client to the Tachyon-Mesh cluster.
///
/// Protobuf service stubs will be added in a future change once the Tachyon
/// `.proto` definitions are published. This module owns the channel lifecycle
/// and will expose typed RPCs (viking-context resolve, inference stream…) as
/// they are defined.
pub struct TachyonMeshClient {
    /// The underlying tonic channel, shared across all future service stubs.
    pub channel: Channel,
    /// The endpoint URI this client was built from, kept for logging/reconnect.
    pub endpoint: String,
}

impl TachyonMeshClient {
    /// Create a client with a **lazy** connection.
    ///
    /// The TCP handshake is deferred to the first RPC call, so this never
    /// fails due to a temporarily unavailable Tachyon node. URI validity is
    /// still validated eagerly.
    pub fn new_lazy(endpoint: &str) -> Result<Self> {
        let channel = Channel::from_shared(endpoint.to_owned())
            .with_context(|| format!("invalid Tachyon endpoint URI: {endpoint}"))?
            .connect_lazy();
        info!(endpoint, "TachyonMeshClient created (lazy)");
        Ok(Self {
            channel,
            endpoint: endpoint.to_owned(),
        })
    }

    /// Create a client and **eagerly** verify TCP connectivity.
    ///
    /// Use this in health-check or reconnect paths where you need to confirm
    /// the node is reachable before proceeding.
    pub async fn connect(endpoint: &str) -> Result<Self> {
        info!(endpoint, "connecting to Tachyon-Mesh");
        let channel = Channel::from_shared(endpoint.to_owned())
            .with_context(|| format!("invalid Tachyon endpoint URI: {endpoint}"))?
            .connect()
            .await
            .with_context(|| format!("TCP connection failed: {endpoint}"))?;
        Ok(Self {
            channel,
            endpoint: endpoint.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_uri_is_rejected() {
        assert!(TachyonMeshClient::new_lazy("not a uri !!!").is_err());
    }

    #[test]
    fn valid_http_uri_accepted() {
        let client = TachyonMeshClient::new_lazy("http://localhost:443").unwrap();
        assert_eq!(client.endpoint, "http://localhost:443");
    }

    #[test]
    fn valid_https_uri_accepted() {
        let client = TachyonMeshClient::new_lazy("https://talos-node:443").unwrap();
        assert_eq!(client.endpoint, "https://talos-node:443");
    }
}
