# Implementation Tasks

- [ ] **Task 1: Update Session State**
  - Modify `Session` struct in `faas/orchestrator/src/lib.rs`.
  - Add `consecutive_failures: u32` and `last_failed_target: Option<String>`.

- [ ] **Task 2: Implement Loop Detection**
  - In the tool execution loop, update the counter if `RunCommand` returns a non-zero exit code.
  - Reset the counter if a command succeeds.

- [ ] **Task 3: Situation Report Template**
  - Create a new system prompt template: `templates/situation_report.j2`.
  - The prompt instructions should strictly follow the "Engineering Synthetic Empathy" patterns (Contextual affirmation and Crisis handoff).

- [ ] **Task 4: Implement the Escalation Logic**
  - When the threshold is hit, call the LLM one last time with the `situation_report` prompt.
  - Send the resulting Markdown to the CLI and the `supervisor`.

- [ ] **Task 5: UI Notification**
  - Update `pulsar-cli` to detect the `Escalated` status and display a high-visibility warning to the developer: `⚠️ RABBIT HOLE DETECTED. Handing over Situation Report...`.