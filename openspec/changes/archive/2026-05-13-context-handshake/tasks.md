# Implementation Tasks

- [x] **Task 1: Add Tool Definition**
  - In `faas/orchestrator/src/lib.rs`, add the `SubmitPlan` variant to the `ToolCall` enum, implementing the strict Handshake JSON schema.

- [x] **Task 2: State Machine Update**
  - Enforce the `Planning` state. If the LLM tries to call `RunCommand` while in `Planning`, intercept it, return a local error to the LLM saying *"You must use submit_plan first"*, and do not execute the command.

- [x] **Task 3: Integration with Suspension Logic**
  - Wire `SubmitPlan` to call the same `suspend()` serialization method created in Phase 14.
  - Emit a specific `{"type": "handshake", "plan": {...}}` payload via the `workspace_bridge`.

- [x] **Task 4: CLI Dashboard Rendering**
  - In `pulsar-cli`, intercept the `handshake` event.
  - Use `crossterm` or `ratatui` to render a clean, boxed UI displaying the Goal, Branch, Files, and Steps.
  - Read `stdin` for approval or correction.

- [x] **Task 5: Context Re-affirmation (Prompting)**
  - Update the prompt templates so that once the plan is approved and the agent enters the `Acting` loop, the approved plan is pinned to the top of the context window as immutable truth: `[APPROVED PLAN: ...]`.