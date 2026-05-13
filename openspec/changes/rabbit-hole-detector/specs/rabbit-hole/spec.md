# Specification: Situation Report Protocol

## Requirement: Threshold Enforcement
The Orchestrator SHALL NOT exceed 3 consecutive failed attempts at the same logical task before escalating.

## Requirement: Report Generation
The `faas/orchestrator` SHALL generate a Markdown report during escalation. This report MUST be transmitted back to the CLI via the WebSocket bridge.

## Requirement: Termination
Upon escalation, the current FaaS instance MUST terminate its automated loop and wait for a new directive or a human intervention signal.

```rust
// Internal State Update
pub enum AgentStatus {
    Thinking,
    Acting,
    Escalated { report: String },
    Finished,
}
```