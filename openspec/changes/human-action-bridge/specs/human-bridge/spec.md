# Specification: Suspension Protocol

## Requirement: Tool Definition
The inference engine SHALL expose a tool designed exclusively for physical/qualitative delegation.
```json
{
  "name": "request_human_action",
  "description": "Suspend execution and ask the human developer to perform a physical task, provide a secret, or make a visual judgment.",
  "parameters": { "instruction": "string", "requires_text_feedback": "boolean" }
}
```

## Requirement: Stateless Resumption
The `faas/orchestrator` MUST be able to fully rehydrate its context from the `kv-partition` without losing the execution trace prior to the suspension.

## Requirement: CLI Interaction
The `pulsar-cli` SHALL prevent any other agent commands from executing in the current worktree until the pending human action is resolved.