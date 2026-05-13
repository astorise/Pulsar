# Implementation Tasks

- [ ] **Task 1: Update Orchestrator WIT**
  - Replace the old `kv-partition` import in `faas/orchestrator/wit/world.wit` with the new V2 `resource table` definition.

- [ ] **Task 2: Observation Data Structure**
  - Define `McpObservation` in `faas/orchestrator/src/lib.rs` with `serde`.
  - Create a helper to convert `u64` timestamps to big-endian strings/bytes for consistent B-Tree sorting.

- [ ] **Task 3: Implement Table-Aware Broadcast**
  - Implement `broadcast_to_swarm(content: &str)`.
  - Instantiate `Table::new("mcp_observations")` and call `set()`.

- [ ] **Task 4: Implement Paginated Sync**
  - Implement `fetch_recent_intel()`.
  - Use `get_range` within a `while` loop, passing `limit = 50` and incrementing `offset`.
  - Break the loop when the returned chunk is empty or when the accumulated context string reaches a safe maximum length (e.g., 2000 tokens).

- [ ] **Task 5: Refine Prompt Injection**
  - Update `build_inference_prompt` to include a new section: `### RECENT SWARM INTELLIGENCE`.
  - Inject the formatted results of `fetch_recent_intel()`.