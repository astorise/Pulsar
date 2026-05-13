# Design: The Handshake Protocol

## 1. The Planning State
The FaaS state machine gets a new initial state: `Planning`. 
When a new session starts, the Orchestrator's system prompt dictates: *"You must use the `submit_plan` tool before any other tool."*

## 2. The Handshake Payload
The `submit_plan` tool requires a strict JSON schema:
```json
{
  "goal_understanding": "string",
  "active_branch": "string",
  "target_files": ["string"],
  "execution_steps": ["string"],
  "risk_assessment": "string"
}
```

## 3. Suspension and UX
When `submit_plan` is called:
1. The Orchestrator serializes its state to the `kv-partition` (reusing Phase 14 logic).
2. It emits a `Handshake` WebSocket event to the CLI.
3. The CLI renders a formatted, readable dashboard of the plan.
4. The CLI prompts: `[Enter] to Approve, or type a correction:`

## 4. Resumption
If the user approves, the FaaS is re-invoked, transitions to the `Acting` state, and executes Step 1. If the user corrects it, the feedback is appended to the trace, and the FaaS must generate a new `submit_plan`.