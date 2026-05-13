# Implementation Tasks

- [ ] **Task 1: Update State Machine**
  - In `faas/orchestrator/src/lib.rs`, add `Recalling` to the `AgentStatus` enum.

- [ ] **Task 2: Implement Skill Discovery**
  - In `faas/orchestrator/src/lib.rs`, modify the `submit_input` function.
  - Before fetching the Viking context, call `workspace_bridge::webdav_propfind(&session_id, ".pulsar/skills")`.
  - Handle gracefully the case where the directory does not exist yet (`NOT_FOUND`).

- [ ] **Task 3: Basic Skill Matching**
  - Create a helper function `match_skill(prompt: &str, available_skills: &[String]) -> Option<String>`.
  - Implement a basic heuristic (e.g., tokenizing the prompt and counting keyword overlaps with the skill filenames).

- [ ] **Task 4: Fetch and Inject**
  - If a matching skill is found, fetch it via `workspace_bridge::webdav_get`.
  - Update `build_inference_prompt` signature to accept an `Option<&str>` for `past_skill`.
  - Append the skill content to the final LLM prompt under a clearly marked section: `--- PAST EXPERIENCE (SKILL) ---`.