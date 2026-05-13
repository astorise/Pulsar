# Specification: Table-Based MCP Protocol

## Requirement: Resource Management
The Orchestrator MUST use the `tachyon:ai/kv-partition` interface's `resource table` to isolate MCP data. The Wasmtime garbage collector will handle the `Drop` implementation to free handles.

## Requirement: Paginated Range Queries (Anti-OOM)
The Orchestrator SHALL leverage `get-range` to retrieve observations. It MUST supply a `limit` to prevent allocating massive lists in Wasm memory and use an `offset` loop to iterate through the temporal window.

## Requirement: MCP Data Structures
```rust
#[derive(Serialize, Deserialize)]
struct McpObservation {
    pub author_session: String,
    pub timestamp: u64,
    pub related_files: Vec<String>,
    pub content: String,
}
```