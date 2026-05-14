# Proposal: Orchestrator Security Hardening & Input Validation

**Date**: 2026-05-15
**Status**: Active
**Category**: Security & Reliability

## Context & Motivation
An independent security audit identified critical vulnerabilities within the Pulsar FaaS orchestrator. The current implementation trusts LLM-generated tool calls implicitly. Specifically, operations like `EditFile` and `RunCommand` lack boundary checks, opening the door for Path Traversal (e.g., accessing `/etc/passwd`) and Command Injection. Furthermore, the orchestrator's hot path contains numerous `unwrap()` and `expect()` calls, leading to silent WebAssembly panics instead of triggering the swarm's self-healing mechanisms.

## Proposed Changes
1. **Viking Context Guard**: Implement strict path validation for all file I/O operations. All paths must be resolved relative to the defined workspace root, expressly rejecting directory traversal attempts (`../`) and unauthorized absolute paths.
2. **Command Injection Guard**: Sanitize all shell executions in `RunCommand`. We will introduce a strict command allowlist and utilize `shlex` (or equivalent shell-escaping logic) to safely tokenize arguments.
3. **Panic Eradication**: Replace all `unwrap()`/`unwrap_or_default()` instances in the execution flow with robust error handling (using `anyhow` or `thiserror`).
4. **Rabbit Hole Trigger**: Ensure that surface-level errors (like an invalid path or forbidden command) are passed back to the LLM as textual context, forcing the agent to evaluate the failure and adapt its plan (Rabbit Hole Detection).

## Impact
* **Critical Security Fix**: Mitigates arbitrary code execution (ACE) and arbitrary file read/write vulnerabilities.
* **Resilience**: Prevents abrupt Wasm component crashes, maintaining the stability of the Tachyon host's worker threads.
* **Agent Autonomy**: Transforms system errors into actionable feedback for the AI, significantly improving the swarm's self-correction capabilities.
