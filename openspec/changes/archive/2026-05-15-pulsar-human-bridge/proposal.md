# Proposal: The Human Bridge (Zero-Cost Asynchronous Suspension)

**Date**: 2026-05-21
**Status**: Active
**Category**: Core Architecture & Safety

## Context & Motivation
Pulsar is built on three dogmas: Context Handshake, Rabbit Hole Detection, and the Human Bridge. While the first two are successfully implemented, the orchestrator currently executes fully autonomously once a plan is generated. 
For high-impact operations (e.g., deleting files, deploying infrastructure, executing complex shell scripts), AI must act as a colleague, not an untethered autonomous script. We need a mechanism to suspend an agent's execution, request human approval, and resume operation without blocking host threads or consuming compute resources while waiting.

## Proposed Changes
1. **WIT Contract**: Introduce `tachyon:mesh/human-bridge` allowing Wasm agents to yield execution and request human validation for a specific plan.
2. **Host Async Suspension**: Leverage Wasmtime's asynchronous execution capabilities. When the `request-approval` host function is called, the Rust host pauses the Wasm future, registers the pending approval in the `redb` store, and frees the worker thread.
3. **Approval API**: Expose a REST/WebSocket endpoint on the Tachyon host (`/api/human-bridge/...`) to allow human operators (via CLI or UI) to review, approve, modify, or reject pending requests.
4. **Orchestrator Integration**: Modify `faas/orchestrator` to automatically invoke the Human Bridge before executing any `RunCommand` or `EditFile` tool call that passes a defined impact threshold.

## Impact
* **Absolute Production Safety**: Destructive or critical actions are guaranteed to have a human-in-the-loop.
* **Compute Efficiency**: Suspended agents consume zero CPU cycles and minimal RAM while waiting for human input, allowing the swarm to scale massively.
* **Dogma Completion**: Fulfills the final foundational philosophy of the Pulsar ecosystem.