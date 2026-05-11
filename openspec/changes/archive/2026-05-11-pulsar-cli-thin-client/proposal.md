# Proposal: Pulsar CLI as a Thin Client

## Problem Statement
The original `pulsar-cli` was designed as a heavy orchestrator. We are pivoting to a centralized intelligence model where `faas/orchestrator` holds the agentic loop. We need to replace the local CLI with a lightweight bridge that simply connects the user's local filesystem and terminal to the Tachyon-hosted orchestrator.

## Vision
The new `pulsar-cli` is a hyper-fast, stateless Rust daemon. It has three simple jobs:
1. **WebDAV Server:** It mounts the current working directory (`.`) and exposes it as a WebDAV share on a local port, allowing the remote FaaS Orchestrator to read and write files as if they were local.
2. **WebSocket Relay:** It maintains a persistent WebSocket connection to the `faas/orchestrator` deployed on the Tachyon cluster.
3. **Terminal UI:** It provides a simple input loop (REPL) that sends text to the WebSocket and prints the incoming tokens (agent thoughts and responses) to the screen.