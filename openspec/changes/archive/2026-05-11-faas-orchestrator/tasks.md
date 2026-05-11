# Implementation Tasks

- [x] **Task 1: Project Scaffolding**
  - Create `faas/orchestrator/` project.
  - Add dependencies: `tachyon-sdk`, `reqwest` (for WebDAV), `serde`, `serde_json`, and `anyhow`.

- [x] **Task 2: WebDAV Client Implementation**
  - Implement a basic WebDAV wrapper to `GET`, `PUT`, and `PROPFIND` files from the CLI's workspace.

- [x] **Task 3: The Agentic State Machine**
  - Implement the core loop: `Prompt -> Inference -> Tool Call -> Observation`.
  - Tools to implement: `read_viking_context`, `edit_file` (via WebDAV), `run_command` (via WebSocket).

- [x] **Task 4: Integration with Viking Context**
  - Connect the orchestrator to the `viking-context` FaaS to pull semantic skeletons before reading raw files.

- [x] **Task 5: Async Learning Trigger**
  - Implement the hook that sends the execution trace to `skill-extractor` upon task completion.
