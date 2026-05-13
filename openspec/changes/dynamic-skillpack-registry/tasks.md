# Implementation Tasks

- [ ] **Task 1: Define `SkillDef` Rust Struct**
  - In `faas-sdk` (or shared types), define a struct with `name`, `description_embedding: Vec<f32>`, `system_prompt`, and `allowed_tools`.
  - Implement `serde` (Serialize/Deserialize) using `rmp-serde` (MessagePack) for maximum speed.

- [ ] **Task 2: Build the `skillify` CLI Command**
  - In `pulsar-cli`, add a command that traverses `.pulsar/skills/**/*.md`.
  - Use the `gray_matter` crate to split YAML frontmatter from Markdown body.
  - Call the local Embedding model to compute the description vector.

- [ ] **Task 3: RedDB Batch Injection**
  - Create the `pulsar_skill_registry` table.
  - The CLI pushes the compiled `SkillDef` MessagePack blobs into this table.

- [ ] **Task 4: Implement Semantic Router in Supervisor**
  - Update `faas/supervisor/src/lib.rs`.
  - Before delegating a sub-task, embed the sub-task text.
  - Load the `pulsar_skill_registry` table, compute cosine similarity for all skills, and pick the one with the highest score ($> 0.80$).

- [ ] **Task 5: Dynamic FaaS Instantiation**
  - Modify the FaaS spawn logic so the Supervisor passes the `skill_name` as an environment variable to the generic `faas/orchestrator`.
  - The generic Orchestrator boots, reads its specific `SkillDef` from RedDB using its name, and sets its own System Prompt.