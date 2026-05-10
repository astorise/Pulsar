# Proposal: Pulsar CLI Core Orchestrator

## Problem Statement
We need a local entry point (replacing tools like Claude Code) that lives in the developer's terminal. It must orchestrate the AI loop, handle local Git operations, and communicate with the Tachyon FaaS backend without sending massive uncompressed codebase payloads.

## Vision
The `pulsar-cli` will be an asynchronous Rust binary. Its core loop is:
1. **Listen:** Accept user input via a terminal REPL.
2. **Contextualize:** Interrogate the remote `faas/viking-context` via gRPC to pull L0/L1 contexts.
3. **Infer:** Send the problem to the inference Tier (Tier 1 local or Tier 2 remote).
4. **Act:** Execute the requested MCP tools locally (e.g., editing files, running tests).

This initial implementation will focus on scaffolding the CLI, setting up the `tokio` async runtime, and defining the gRPC client structure.