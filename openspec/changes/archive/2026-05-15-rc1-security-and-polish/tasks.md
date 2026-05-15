# Tasks: RC1 Security & Polish

- [x] Modify `faas/orchestrator/wit/world.wit` to use `args: list<string>` and return `exit-code` for `websocket-command`.
- [x] Remove duplicate WIT files (delete top-level `wit/ai/inference.wit` if unreferenced).
- [x] Refactor `faas/orchestrator/src/lib.rs` to evaluate command success using the new explicit exit code.
- [x] Rewrite `PathGuard::validate` to use `std::path::Component` resolution instead of substring matching.
- [x] Audit `faas/orchestrator/src/lib.rs` and inject `PathGuard::validate` into all read operations (`read_viking_context`, `ask_lsp`, etc.).
- [x] Update `is_allowed_command` to implement the strict verb sub-allowlist for `git` and block `-c`/`--exec-path`.
- [x] Rewrite `tests/wasm_e2e_runner.rs` to use `wasmtime::Component::from_file` instead of testing native Rust modules.
- [x] Delete or archive the orphaned `core-host/` directory if its features are fully handled by Tachyon.
- [x] Create a `SECURITY.md` at the repository root defining the threat model (LLM vs. Host vs. Workspace).
