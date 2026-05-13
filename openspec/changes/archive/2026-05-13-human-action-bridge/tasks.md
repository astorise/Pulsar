# Implementation Tasks

- [x] **Task 1: Orchestrator Tooling**
  - In `faas/orchestrator/src/lib.rs`, add `RequestHumanAction { instruction: String, requires_feedback: bool }` to the `ToolCall` enum.

- [x] **Task 2: State Serialization**
  - Ensure the `AgentSession` struct derives `serde::Serialize` and `serde::Deserialize`.
  - Implement a `suspend()` method that writes the JSON state to `workspace_bridge::kv_set`.

- [x] **Task 3: Orchestrator Entrypoint Update**
  - Modify the FaaS `export` entrypoint. Instead of just `start_session`, add a `resume_session(session_id: String, human_feedback: String)` function.
  - This function loads the state from KV, appends a `UserObservation(human_feedback)`, and enters the `Acting` loop.

- [x] **Task 4: Update CLI Protocol**
  - In `pulsar-cli`, handle the new `Suspend` server message.
  - Display the instruction prominently in the terminal/editor.
  - Block stdin for normal chat, prompting specifically for the action feedback.
  - Upon enter, send a `ResumeRequest` back over the WebSocket.

- [x] **Task 5: Meta-Prompt Update**
  - Add to the system prompt: *"If you need to verify a physical device state, read an email, or check visual aesthetics, DO NOT guess. Use `request_human_action` to ask the developer. You will be suspended and woken up when the task is done."*