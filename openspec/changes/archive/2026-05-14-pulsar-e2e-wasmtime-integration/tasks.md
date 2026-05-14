# Tasks: E2E Wasmtime Integration

- [x] Add `wasmtime` and `wasmtime-wasi` as `dev-dependencies` to the Pulsar workspace.
- [x] Create the `tests/` directory and `tests/wasm_e2e_runner.rs` file.
- [x] Setup the `wasmtime::Engine`, `Linker`, and `Store` with WASI context initialized to a temporary virtual directory.
- [x] Implement the `bindgen!` macro in the test harness to generate host traits from the downloaded OCI WIT contracts.
- [x] Implement the dummy mock state for `tachyon:mesh/graph`, `tachyon:mesh/kv-partition`, and `tachyon:ai/inference`.
- [x] Write `test_path_traversal_blocked` asserting WASI filesystem confinement.
- [x] Write `test_successful_skill_escalation` asserting the retry logic against multiple mock tiers.
- [x] Write `test_rabbit_hole_detection` asserting graceful error handling and prompt adjustment.
- [x] Update the `.github/workflows/test.yml` to ensure `cargo test --test wasm_e2e_runner` runs the newly compiled Wasm components natively in CI.
