# Implementation Tasks

- [x] **Task 1: Update Workspace Bridge WIT**
  - Modify `faas/orchestrator/wit/world.wit` to add the `lsp-hover` function to the `workspace-bridge` interface.

- [x] **Task 2: Update CLI Protocol**
  - In `pulsar-cli/src/protocol.rs`, add `LspHoverRequest` to `ServerMessage` (FaaS to CLI) and `LspHoverResponse` to `ClientMessage` (CLI to FaaS).
  - Add logic in `pulsar-cli` to handle the blocking nature of the WIT call (using a correlation ID/channels to wait for the client's response).

- [x] **Task 3: VS Code Hover Execution**
  - In `pulsar-vscode/src/PulsarClient.ts`, listen for the `lsp_hover_request` message type.
  - Implement the call to `vscode.commands.executeCommand('vscode.executeHoverProvider', ...)` using the provided path and coordinates.
  - Format the returned `Hover[]` data into a single Markdown string and send it back via `lsp_hover_response`.

- [x] **Task 4: Add AskLsp Tool to Orchestrator**
  - In `faas/orchestrator/src/lib.rs`, add `AskLsp` to the `ToolCall` enum.
  - Implement the execution match arm: call `workspace_bridge::lsp_hover` and record the returned signature in the observation trace.

- [x] **Task 5: Refine the Meta-Prompt**
  - Update `build_inference_prompt` in the orchestrator.
  - Add the instruction: *"If you encounter an external function or type and are unsure of its signature or parameters, DO NOT guess. Use the `ask_lsp` tool to hover over it and read the exact documentation from the compiler."*