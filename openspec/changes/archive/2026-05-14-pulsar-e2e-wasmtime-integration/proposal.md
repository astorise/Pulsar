# Proposal: End-to-End Wasmtime Integration and Isolation Testing

**Date**: 2026-05-17
**Status**: Active
**Category**: Testing & Security

## Context & Motivation
The independent security audit highlighted a critical blind spot: Pulsar lacks true WebAssembly (Wasm) integration tests. Currently, `#[test]` modules compile and run as native Rust code. This means the boundaries of the Wasm Component Model, the host imports (Tachyon-Mesh), and the sandboxed execution environment are never explicitly tested in CI. The claim of "total isolation" is therefore theoretical.

To transition from an aspirational prototype to a production-grade swarm, we must mathematically prove that the orchestrator behaves correctly when confined within a strict Wasm runtime.

## Proposed Changes
1. **Wasmtime Test Harness**: Introduce a dedicated `tests/` directory at the Pulsar workspace root using the `wasmtime` crate. This harness will compile the orchestrator to `wasm32-wasip1` and instantiate it inside a real Wasmtime engine during `cargo test`.
2. **Mocking the Tachyon Host**: Implement dummy Rust host functions in the test harness that satisfy the `tachyon:mesh` and `tachyon:ai` WIT imports. This allows us to simulate semantic graph queries, KV-cache hits, and LLM responses natively without needing the actual Tachyon host.
3. **E2E Security Validation**: Write explicit tests asserting that the Wasm component *cannot* escape its virtualized filesystem (proving the effectiveness of the Phase 2 PathGuard) and correctly triggers the Tier 2 escalation loop (proving the Phase 3 implementation).

## Impact
* **Verifiable Security**: Proof that Path Traversal is impossible at the runtime level.
* **Component Integrity**: Prevents future regressions where an agent might accidentally depend on host capabilities not defined in the WIT contract.
* **Audit Resolution**: Fully resolves the "MicroVM isolation claim unverifiable" finding from the Claude Code audit.
