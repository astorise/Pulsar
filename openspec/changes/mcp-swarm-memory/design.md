# Design: Paginated Table Memory Architecture

## 1. Table Definitions
The Orchestrator interacts with a specific table resource:
- **Table Name:** `mcp_observations`
- **Key Format:** Big-endian `u64` timestamp (to maintain B-Tree sorting).
- **Value Format:** JSON-serialized `McpObservation`.

## 2. The Broadcast Flow (Write)
When an agent recovers from a failure or reaches a milestone:
1. `let table = Table::new("mcp_observations");`
2. `table.set(current_timestamp_be_bytes, observation_json)`.

## 3. The Synchronization Flow (Paginated Read)
At the start of every "Think" cycle, the Orchestrator fetches recent intel safely:
```rust
let mut offset = 0;
let limit = 50; // Strict limit to prevent OOM
let mut observations = Vec::new();

loop {
    let chunk = table.get_range(start_ts, end_ts, limit, offset)?;
    if chunk.is_empty() { break; }
    
    // Parse and accumulate up to a safe threshold...
    
    offset += limit;
}
```
The parsed observations are injected into the LLM prompt.