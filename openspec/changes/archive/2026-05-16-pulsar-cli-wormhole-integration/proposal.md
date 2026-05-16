# Proposal: WebDAV NAT Traversal via Wormhole

## Context
The Pulsar CLI has been migrated to Node.js and successfully spawns a local WebDAV server to act as a shared workspace for remote Wasm agents. However, developers working behind standard ISP routers or corporate firewalls are protected by NAT, making the local WebDAV address (`127.0.0.1` or internal LAN IP) unreachable for Tachyon-Mesh orchestrators deployed in the cloud.

## Objective
Integrate the `@tachyon-mesh/wormhole` library directly into the `pulsar-cli` to securely expose the local WebDAV TCP port through a public Tachyon edge relay.

## Rationale
* **Zero Configuration:** Developers will not need to configure complex port-forwarding rules on their home or corporate routers.
* **Security:** Wormhole uses QUIC and mutual TLS (mTLS) to secure the tunnel to the relay.
* **Seamless Workflow:** The CLI will dynamically bind a local WebDAV port, open the Wormhole tunnel, and send the *public* relay URL to the orchestrator instead of the local one.