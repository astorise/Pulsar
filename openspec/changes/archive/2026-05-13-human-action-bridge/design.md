# Design: Event-Driven Suspension

## 1. The FaaS Suspension (State Serialization)
The `faas/orchestrator` state machine is updated. When the LLM outputs `ToolCall::RequestHumanAction { instruction }`:
1. The Orchestrator serializes its `AgentSession` (trace, variables, token usage) into JSON.
2. It saves it to `kv-partition` at `v1:sessions:suspended:{session_id}`.
3. It emits a WebSocket message `{"type": "suspend", "instruction": "..."}`.
4. The FaaS process exits (`return Ok(())`).

## 2. The CLI User Experience
The `pulsar-cli` (or VS Code extension) intercepts the `suspend` message.
It renders a distinct UI block:
> 🛑 **Pulsar is paused.**
> **Request:** "Please connect the Meta Quest 3 via USB and ensure SideQuest detects it."
> `[ Text Input for Feedback ]` `[ Confirm Action ]`

## 3. The FaaS Resumption (Event Wakeup)
When the user clicks "Confirm", the CLI calls the Tachyon host with a new command: `resume_session(session_id, user_feedback)`.
The Tachyon host spins up a fresh `faas/orchestrator` instance. The first thing this instance does is read `v1:sessions:suspended:{session_id}` from the `kv-partition`, append the `user_feedback` to the trace, and trigger the `Think` cycle again.