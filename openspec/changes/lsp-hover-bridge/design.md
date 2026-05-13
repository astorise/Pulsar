# Design: LSP Bridge Architecture

## 1. The FaaS Orchestrator
The orchestrator gets a new tool: `AskLsp { path: String, line: u32, character: u32 }`.
When the LLM selects this tool, the FaaS calls a new `workspace-bridge` WIT function: `lsp-hover`. 

## 2. The WebSocket Protocol
We add an asynchronous request/response flow to the WebSocket channel:
- **Outbound:** `{ "type": "lsp_hover_request", "request_id": "...", "path": "...", "line": 10, "character": 5 }`
- **Inbound:** `{ "type": "lsp_hover_response", "request_id": "...", "content": "```rust\npub fn new() -> Self\n```" }`

## 3. VS Code Extension (The Execution)
When `pulsar-vscode` receives an `lsp_hover_request`:
1. It resolves the `path` to a local `vscode.Uri`.
2. It constructs a `vscode.Position(line, character)`.
3. It calls the native VS Code API: `vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider', uri, position)`.
4. It extracts the `vscode.MarkdownString` contents from the first hover result and sends it back in the `lsp_hover_response`.