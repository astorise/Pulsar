# Design: Skill Injection Architecture

## 1. The Recalling State
We expand the `AgentStatus` enum in `faas/orchestrator` to include `Recalling`.
When a user submits a prompt, the orchestrator temporarily enters this state.

## 2. WebDAV Propfind for Skills
Using the existing `workspace_bridge::webdav_propfind` function, the orchestrator lists the contents of the `.pulsar/skills/` directory.

## 3. Heuristic Filtering & Injection
For Phase 1, we will use a simple heuristic:
- The orchestrator downloads the names of all `.md` skills.
- It performs a keyword intersection between the user's prompt and the skill filenames (which are sanitized task descriptions).
- The best matching skill is fetched via `workspace_bridge::webdav_get`.
- The markdown content is appended to the `build_inference_prompt` function template.