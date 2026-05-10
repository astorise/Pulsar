use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backend: BackendConfig,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    /// gRPC endpoint of the Tachyon-Mesh cluster (e.g. `http://localhost:443`).
    pub tachyon_endpoint: String,
    /// Tier-1 inference mode: `"local"` uses the unified-memory/eGPU node.
    pub tier1_inference: String,
    /// Tier-2 inference endpoint (e.g. `grpc://talos-node:443`).
    pub tier2_inference: String,
}

impl Config {
    /// Load config from `pulsar.toml`.
    /// Looks in the current working directory first, then `$HOME`.
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading config: {}", path.display()))?;
        Self::parse(&text)
    }

    /// Parse a TOML string into a `Config`. Separated for unit testing.
    pub fn parse(s: &str) -> Result<Self> {
        toml::from_str(s).context("invalid pulsar.toml")
    }

    fn path() -> Result<PathBuf> {
        let local = PathBuf::from("pulsar.toml");
        if local.exists() {
            return Ok(local);
        }
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("HOME / USERPROFILE not set — cannot locate pulsar.toml")?;
        Ok(PathBuf::from(home).join("pulsar.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
[backend]
tachyon_endpoint = "http://localhost:443"
tier1_inference  = "local"
tier2_inference  = "grpc://talos-node:443"
"#;

    #[test]
    fn parse_valid_config() {
        let cfg = Config::parse(VALID).unwrap();
        assert_eq!(cfg.backend.tachyon_endpoint, "http://localhost:443");
        assert_eq!(cfg.backend.tier1_inference, "local");
        assert_eq!(cfg.backend.tier2_inference, "grpc://talos-node:443");
    }

    #[test]
    fn parse_missing_field_fails() {
        let toml = "[backend]\ntachyon_endpoint = \"http://localhost:443\"\n";
        assert!(Config::parse(toml).is_err());
    }

    #[test]
    fn parse_invalid_toml_fails() {
        assert!(Config::parse("not { valid ] toml").is_err());
    }
}
