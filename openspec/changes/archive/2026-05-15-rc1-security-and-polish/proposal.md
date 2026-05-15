# Proposal: Release Candidate 1 (RC1) Security Hardening & E2E Polish

**Date**: 2026-05-18
**Status**: Active
**Category**: Security & Testing

## Context & Motivation
Following the second independent security audit, the Pulsar swarm has proven structurally sound, but critical execution boundaries remain porous. Specifically:
1. **Command Injection Bypass (H1)**: Flattening shell arguments into a single string allows the LLM to escape the orchestrator's sandbox if the host blindly re-parses it.
2. **Path Traversal Leaks (H2/H3)**: The `PathGuard` relies on naive string matching (`../`) and fails to protect read operations, leaving the swarm vulnerable to exotic encoding bypasses.
3. **Wasmtime E2E Illusion (M2)**: The current integration harness tests native Rust code, failing to prove true Wasm component isolation.

## Proposed Changes
1. **Strict Boundary Contracts**: Modify the WIT interfaces (`websocket-command`) to accept `list<string>` (argv) and return explicit integer exit codes, eliminating heuristic parsing entirely.
2. **Ironclad PathGuard**: Rewrite `PathGuard` to use strict path component resolution (`std::path::Component::Normal`) instead of substring matching, and apply it uniformly across all read/write operations.
3. **True Wasm Instantiation**: Refactor `tests/wasm_e2e_runner.rs` to compile `wasm32-wasip1`, load the actual `.wasm` binary using `wasmtime::Component::from_file`, and assert its confinement.
4. **Git Action Sandbox**: Restrict the `git` command allowlist to specific safe verbs (e.g., `status`, `commit`, `push`) while outright blocking configuration overrides (`-c`, `--exec-path`).
5. **Deduplication & Hygiene**: Consolidate redundant WIT definitions, archive orphaned host code (`core-host`), and add a formal `SECURITY.md` threat model.

## Impact
* **Production Readiness**: Clears all High and Medium security findings, achieving the promised "total isolation" necessary for a v0.1.0 release.
* **Provable Confinement**: The CI pipeline will mathematically verify that the LLM-generated code cannot break out of the WASI boundaries.