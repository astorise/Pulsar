# Proposal: Secrets Management, CI Polish, and Repository Hygiene

**Date**: 2026-05-18
**Status**: Active
**Category**: Security & Documentation

## Context & Motivation
While the core architectural and execution vulnerabilities have been patched, the independent security audit highlighted several operational and hygiene issues. Specifically:
1. Hardcoded or plaintext credentials (`workspace_token`) reside in a global `SESSIONS` Mutex, exposing them to memory dumps or compromised Wasm agents.
2. The CI pipeline is incomplete, ignoring half of the FaaS crates and lacking automated secret scanning.
3. The repository lacks a formal `LICENSE`, and the `README.md` creates a disconnect by blending aspirational features with current realities.
4. "Phantom specs" (`knowledge-graph`, `token-killer`) exist without implementation, creating architectural confusion.

## Proposed Changes
1. **Secrets Management**: Refactor `faas/orchestrator` to stop holding plaintext tokens in memory. Rely on Tachyon's `system-faas-tde` (Transparent Data Encryption) or a host-injected secret mechanism via the WASI interface.
2. **CI Expansion**: Update `.github/workflows/test.yml` to enforce `cargo clippy` and `cargo test` across all 8 workspace members. Introduce `gitleaks` for automated secret scanning.
3. **Documentation Realignment**: Rewrite the root `README.md` to clearly separate the "Long-term Vision" from the "Current Implementation". Add a standard MIT `LICENSE`.
4. **Spec Pruning**: Move unimplemented, orphaned OpenSpecs (`knowledge-graph`, `token-killer`) to an `/archive` directory to freeze their state until they are actually scheduled for development.

## Impact
* **Memory Security**: Prevents credential theft if the Wasm boundary is theoretically breached.
* **Maintainability**: Clear, honest documentation sets correct expectations for contributors.
* **CI Reliability**: Guarantees that no crate is left behind during code quality checks and prevents accidental token leaks.