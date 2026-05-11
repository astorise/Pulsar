# Design: faas/orchestrator

## Architecture Overview
This component is the core state machine of the Pulsar ecosystem.



**Execution Flow:**
1. **Handshake:** The CLI connects via WebSocket. The Orchestrator receives the WebDAV URL for the local workspace.
2. **Reconnaissance:** The Orchestrator queries `viking-context` (L0/L1) to map the codebase structure.
3. **The Loop (Think-Act-Observe):**
   - **Think:** Calls `inference-gateway` (Tier 1 or Tier 2) to decide on the next action.
   - **Act:** Performs actions like `read_file` or `write_file` via WebDAV or requests local command execution via WebSocket.
   - **Observe:** Captures the output (e.g., compiler errors, test results).
4. **Conclusion:** Once the task is done, it sends the final trace to `skill-extractor`.

## Integration Targets
- **WebDAV Client:** The FaaS includes a lightweight WebDAV client to interact with the CLI's exposed file system.
- **WebSocket Server:** The FaaS exposes a WebSocket endpoint for the CLI/VS Code extension.