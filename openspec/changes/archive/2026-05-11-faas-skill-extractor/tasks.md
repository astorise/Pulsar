# Implementation Tasks for AI Agent

- [x] **Task 1: WIT Integration**
  - Create the file `wit/ai/skill-extractor.wit` using the definition from the spec.
  - Update the main Tachyon world to export this new interface and import `tachyon:ai/inference` and `tachyon:mesh/storage-broker`.

- [x] **Task 2: FaaS Scaffolding**
  - Initialize a new Rust WASM project in `faas/skill-extractor/`.
  - Add dependencies: `tachyon-sdk`, `serde`, `serde_json`, and `anyhow`.

- [x] **Task 3: Implement Meta-Prompting Logic**
  - Create a private function `build_meta_prompt(task: &str, trace: &str) -> String`.
  - The prompt must instruct the LLM to format its output strictly as a Markdown document containing:
    1. The context/trigger.
    2. The exact steps to solve it (avoiding the dead-ends seen in the trace).
    3. The specific commands or tools to use.

- [x] **Task 4: Implement Inference Call**
  - In the exported `extract` function, invoke the `tachyon:ai/inference` interface.
  - Pass the generated meta-prompt to the local Qwen 27B model (Tier 2).

- [x] **Task 5: Implement Storage & Response**
  - Parse the LLM's response to extract the markdown content.
  - Sanitize the `task-description` to create a valid filename (e.g., replacing spaces with hyphens).
  - Use `tachyon:mesh/storage-broker` to write the content to `.pulsar/skills/<filename>.md`.
  - Return the `extraction-response` record.
