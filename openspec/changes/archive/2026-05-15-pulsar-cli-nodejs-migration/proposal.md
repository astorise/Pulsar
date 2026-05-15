# Proposal: Migrate Pulsar CLI to Node.js

## Context
Following the recent architectural Front-End pivot, the local client tooling for the Pulsar Swarm needs to be closer to the web/editor ecosystem. Currently, `pulsar-cli` is implemented in Rust. While performant, maintaining the CLI in Rust creates an unnecessary boundary when integrating tightly with `pulsar-vscode`, which is built to run natively in web and desktop VS Code environments.

## Objective
Rewrite the `pulsar-cli` in TypeScript/Node.js to serve as the unified local bridge. This Node.js CLI will handle the WebDAV workspace sharing, the WebSocket connection to the Tachyon orchestrator, and the Human Bridge interaction layer, ultimately allowing the VS Code extension to directly import or seamlessly spawn these capabilities.

## Rationale
* **Ecosystem Alignment:** TypeScript aligns perfectly with the VS Code extension ecosystem, allowing for shared interfaces/types for the WebSocket protocol.
* **WebDAV Ecosystem:** Excellent Node.js libraries (e.g., `webdav-server`) exist to rapidly expose the local worktree.
* **Future Encapsulation:** A Node.js CLI can eventually be bundled directly inside the VS Code extension using `esbuild`, eliminating the need for a separate binary installation on the developer's machine.