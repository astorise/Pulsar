# Design: Rabbit Hole Detection Logic

## 1. Failure Tracking
The `AgentSession` state in `faas/orchestrator/src/lib.rs` is updated to include a `failure_counter`.
- **Target:** A tuple of `(Command, TargetFile)`.
- **Logic:** Each time a `RunCommand` tool returns an error for the same target, the counter increments.

## 2. Triggering the Situation Report
Once the counter reaches the threshold (3), the Orchestrator's state machine transitions to `Escalated`.
Instead of querying the LLM for the next action, it invokes a specialized internal prompt: `GenerateSituationReport`.

## 3. Situation Report Schema (The Pivot Format)
The report is a standardized Markdown file (`SITUATION_REPORT.md`) containing:
- **Objective:** The initial goal.
- **State of Play:** Current Git branch and modified files.
- **The Blockage:** Why the last command failed (error summary).
- **Attempted Paths:** List of what was tried and why it didn't work.
- **Recommended Next Steps:** What a human or a more powerful model (Tier 3) should do.

## 4. Worktree Preservation
The worktree is kept in its current "broken" state, allowing the human to inspect the intermediate files exactly as the IA left them.