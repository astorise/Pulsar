# Interface Update: tachyon:ai/workspace-bridge

We expand the WIT contract to support LSP queries from the FaaS.

```wit
package tachyon:ai@1.0.0;

interface workspace-bridge {
    // ... existing functions (webdav-*, websocket-command) ...

    /// Requests a virtual hover over a specific coordinate in the user's IDE.
    /// Blocks until the client (VS Code) returns the markdown signature.
    lsp-hover: func(session-id: string, path: string, line: u32, character: u32) -> result<string, string>;
}
```

## Orchestrator Tooling Update
The `faas/orchestrator` must add the following to its `ToolCall` enum:
```rust
AskLsp { path: String, line: u32, character: u32 }
```