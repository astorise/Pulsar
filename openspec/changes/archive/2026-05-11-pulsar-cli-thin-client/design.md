# Design: pulsar-cli (Thin Client)

## Architecture Overview
The CLI uses `tokio` for asynchronous networking, completely detached from any AI inference logic.

**Network Flow:**
- **Inbound (WebDAV):** The CLI spins up an HTTP server (using `hyper` and `dav-server`) on an ephemeral or configured port (e.g., `127.0.0.1:8080`). It exposes the local `cwd`. A secure token is generated at startup.
- **Outbound (WebSocket):** The CLI connects to `ws://<tachyon-cluster>/orchestrator`.
- **Handshake:** Upon connection, the CLI sends an `init` message containing the local WebDAV URL and the authentication token.

## Tunneling (Local to WSL2/Talos)
Since the Tachyon FaaS runs inside Kubernetes (k3s/Talos), the FaaS needs to reach the CLI's WebDAV server. 
- If running on the same machine (WSL2), the CLI passes its Windows host IP.
- If remote (Talos), the CLI could automatically use a reverse tunnel (like ngrok, or a built-in reverse proxy feature of Tachyon Mesh) to make the WebDAV accessible. For Phase 1, we assume a routable network or local WSL2 bridge.