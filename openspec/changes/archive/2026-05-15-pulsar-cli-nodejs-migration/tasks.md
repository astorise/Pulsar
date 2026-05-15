# Implementation Tasks

- [x] **Phase 1: Project Scaffolding**
  - [x] Initialize `pulsar-cli-node` directory with `package.json` and `tsconfig.json`.
  - [x] Setup `esbuild` for fast compilation.
  - [x] Define shared TypeScript interfaces for the WebSocket protocol (matching the Rust Orchestrator ABI).

- [x] **Phase 2: Core Bridges**
  - [x] Implement the WebDAV server module with dynamic token authentication.
  - [x] Implement the WebSocket client module to connect to Tachyon (`PULSAR_ORCHESTRATOR_WS`).
  - [x] Establish the bi-directional message routing between the WS client and the terminal stdout/stdin.

- [x] **Phase 3: Git & Human Bridge**
  - [x] Replicate the `merge-worktree` and `skillify` CLI commands in JS.
  - [x] Implement the Sandbox flow: capture agent diffs, display them via standard output, and prompt for user validation.

- [x] **Phase 4: VS Code Integration & Cleanup**
  - [x] Update `pulsar-vscode/package.json` configurations to point to the new Node.js CLI layer.
  - [x] Deprecate and remove the old Rust `pulsar-cli/` directory from the workspace.
  - [x] Verify that OCI artifact publishing and fetching via `wkg` remains unaffected by the CLI change.
