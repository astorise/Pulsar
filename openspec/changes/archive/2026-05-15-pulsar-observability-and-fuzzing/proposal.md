# Proposal: Wasm Telemetry Bridge and Continuous Fuzzing

**Date**: 2026-05-19
**Status**: Active
**Category**: Observability & Security

## Context & Motivation
Following the successful RC1 security remediation, a few "Day 2" operational technical debts remain from the second security audit:
1. **Wasm Telemetry Blackhole (L3)**: The orchestrator uses the `tracing` crate (`tracing::info!`), but without a subscriber bridging the WebAssembly component to the host, these logs are silently dropped. This severely hinders debugging the swarm in production.
2. **Untested Edge Cases**: While `PathGuard` and `CommandRequest` parsing are now structurally secure, they handle highly variable and potentially malicious LLM outputs. They lack generative fuzz testing to prove they cannot be bypassed by unforeseen Unicode or encoding combinations.

## Proposed Changes
1. **Wasm-to-Host Tracing Bridge**: Implement a custom `tracing` subscriber inside the FaaS orchestrator that forwards log events across the FFI boundary to the Tachyon host using the `wasi:logging/logging` standard (or a custom `tachyon:mesh/telemetry` WIT interface).
2. **Continuous Fuzzing**: Introduce `cargo-fuzz` (libFuzzer) to the repository to continuously bombard the `PathGuard::validate` and command parsing logic with mutated byte sequences.
3. **Static Regex Documentation (L2)**: Document the acceptable use of `unwrap()` on static, pre-compiled Regex initializations to satisfy linter/audit requirements without adding unnecessary runtime overhead.

## Impact
* **Full Stack Observability**: Swarm actions, escalations, and rabbit holes will be visible in the Tachyon host's central logs.
* **Proactive Security**: Fuzzing ensures that no future code changes accidentally reopen path traversal or command injection vulnerabilities.