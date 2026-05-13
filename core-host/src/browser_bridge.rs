use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CdpCommand {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CdpResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

pub fn build_cdp_command(id: u64, method: &str, params: serde_json::Value) -> CdpCommand {
    CdpCommand {
        id,
        method: method.to_string(),
        params,
    }
}

pub fn smolvm_browser_rootfs_script() -> &'static str {
    r#"#!/bin/sh
set -eu
apk add --no-cache chromium xvfb-run ttf-dejavu
"#
}
