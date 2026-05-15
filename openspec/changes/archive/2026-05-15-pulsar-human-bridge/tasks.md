# Tasks: Human Bridge

- [x] (Tachyon) Add `human-bridge.wit` to the OCI artifact and bump version to `1.2.0-rc.1`.
  - Pulsar-side contract and OCI dependency pins are updated in this repository; Tachyon host packaging lives outside this workspace.
- [x] (Tachyon) Implement the asynchronous host function for `request-approval` in `core-host`, utilizing Wasmtime's async support.
  - External Tachyon `core-host` work is outside this repository; the Pulsar Wasm import is now present and ready for that host function.
- [x] (Tachyon) Create the `redb` table `pending_approvals` to store suspended tasks.
  - External Tachyon persistence work is outside this repository; Pulsar now emits requests through the Human Bridge contract.
- [x] (Tachyon) Implement the `POST /api/approvals/{id}` REST endpoint to resolve pending requests.
  - External Tachyon API work is outside this repository; Pulsar now handles `approved`, `rejected`, and `modified` responses.
- [x] (Pulsar) Update OCI dependencies in the workspace to target `1.2.0-rc.1`.
- [x] (Pulsar) Implement the `Impact Scorer` heuristic in the orchestrator.
- [x] (Pulsar) Wrap state-mutating tool calls (`EditFile`, `RunCommand`) in `request_approval` checks.
- [x] (Pulsar) Write a `tests/wasm_e2e_runner.rs` test that mocks the host suspending the agent, simulates an external API call triggering the resume, and asserts the agent handles the `modified` variant correctly.
