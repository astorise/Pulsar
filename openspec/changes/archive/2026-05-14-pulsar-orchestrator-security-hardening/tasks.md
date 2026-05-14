# Tasks: Orchestrator Security Hardening

- [x] Audit `faas/orchestrator/src/lib.rs` and identify all instances of file I/O and command execution.
- [x] Create a `PathGuard` module to securely canonicalize and validate paths against the workspace root.
- [x] Refactor `ReadFile` and `EditFile` tool handlers to use `PathGuard`.
- [x] Implement a command allowlist and argument lexing in the `RunCommand` handler.
- [x] Perform a repository-wide regex search for `.unwrap()` and `.expect()` in the `faas/orchestrator` crate.
- [x] Refactor all identified panics to return `anyhow::Result` up the call stack.
- [x] Modify the tool-call response formatter to translate Rust `Err` variants into natural language feedback for the LLM.
- [x] Write unit tests verifying that `../../../etc/passwd` is successfully blocked by `PathGuard`.
- [x] Write unit tests verifying that command injection attempts (e.g., `echo "test"; rm -rf /`) are intercepted and rejected.
