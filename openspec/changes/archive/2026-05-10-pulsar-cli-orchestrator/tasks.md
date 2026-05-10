# Implementation Tasks

- [x] **Task 1: Project Scaffolding**
  - Update `pulsar-cli/Cargo.toml` with dependencies: `tokio` (full), `tonic` (transport only, no TLS for scaffold), `prost`, `serde`, `serde_json`, `toml`, `anyhow`, `tracing`, `tracing-subscriber`, and `rustyline`.

- [x] **Task 2: CLI Configuration loader**
  - Create `pulsar-cli/src/config.rs` to parse `pulsar.toml` defining the backend endpoints.
  - `Config::parse(&str)` separated from `Config::load()` for unit testability.
  - Tests: valid config, missing field, invalid TOML.

- [x] **Task 3: Basic REPL Loop**
  - Create `pulsar-cli/src/repl.rs`.
  - Synchronous `Repl::run()` called via `tokio::task::block_in_place` in main.
  - Prompts `pulsar> `, persists history, exits on `/exit` / `/quit` / EOF / Ctrl-C.
  - Stub dispatch to orchestrator documented with TODO comment.
  - Test: smoke-test `Repl::new()` in headless environment.

- [x] **Task 4: Tachyon gRPC Client Foundation**
  - Create `pulsar-cli/src/mesh_client.rs`.
  - `TachyonMeshClient::new_lazy` — URI-validated, TCP-deferred (connect on first RPC).
  - `TachyonMeshClient::connect` — eager async connect for health-check paths.
  - Tests: invalid URI rejected, valid HTTP/HTTPS URIs accepted.

- [x] **Task 5: Main Orchestrator Wire-up**
  - Update `pulsar-cli/src/main.rs`: init tracing, load config, create lazy mesh client, start REPL.
  - Stub `pulsar-cli/src/orchestrator.rs` created (Think → Act → Observe skeleton).
