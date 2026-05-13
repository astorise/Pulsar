# Implementation Tasks

- [ ] **Task 1: Add Regex Dependency**
  - In `faas/orchestrator/Cargo.toml`, add `regex = "1"`. (No need for `toml`, we use the already present `serde_json`).

- [ ] **Task 2: Define Data Structures**
  - Create `faas/orchestrator/src/sanitizer.rs`.
  - Use `serde` to define `FilterRule` matching the JSON schema.
  - Load and parse `include_str!("../../filters.json")` into a `std::sync::LazyLock<Vec<FilterRule>>`. *Pre-compile all regexes inside the LazyLock block for O(1) runtime performance.*

- [ ] **Task 3: Implement Engine & Truncation**
  - Implement `pub fn clean_output(command: &str, raw_output: &str) -> String`.
  - Apply ANSI stripping: `Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap().replace_all(...)`.
  - Apply the JSON rules (line by line filtering).
  - Implement `smart_truncate` (keep 30% top, 70% bottom, insert `[... omitted ...]`).

- [ ] **Task 4: Integrate into the Orchestrator**
  - In `faas/orchestrator/src/lib.rs`, import `sanitizer`.
  - In the `ToolCall::RunCommand` match arm, pass the raw string through `sanitizer::clean_output(&command, &raw)` before appending it to the execution trace.