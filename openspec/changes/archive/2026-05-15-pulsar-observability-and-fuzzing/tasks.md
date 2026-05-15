# Tasks: Observability and Fuzzing

- [x] Add `tracing-subscriber` to `faas/orchestrator` dependencies.
- [x] Initialize a Wasm-compatible tracing subscriber at the entrypoint of the orchestrator to forward `tracing::info!` to the host.
- [x] Install `cargo-fuzz` (`cargo install cargo-fuzz`).
- [x] Initialize the fuzzing environment in `faas/orchestrator/fuzz`.
- [x] Write the `fuzz_pathguard` target to test canonicalization escapes.
- [x] Write the `fuzz_command_lexer` target to test argument splitting edge cases.
- [x] Add inline comments (`// SAFETY: ...`) above the static Regex `unwrap()` calls in `sanitizer.rs` to formally address finding L2.
- [x] Add a `fuzz` step to the CI pipeline (optional: run a short 5-minute fuzzing job on nightly builds).
