# Implementation Tasks

- [x] **Task 1: Update Core Host WIT**
  - Update `wit/ai/inference.wit` to include the `embed` function.
  - Update `wit/ai/kv-partition.wit` to include `atomic-swap`.
  - Implement these in `core-host/src/ai_inference.rs` (using Candle's embedding models) and in the RedDB handler.

- [x] **Task 2: Scaffold `faas/cognitive-gc`**
  - Create the new FaaS project.
  - Implement the main loop that fetches the last 100 observations using the paginated `get-range` (Phase 10).

- [x] **Task 3: Implement Clustering Logic**
  - Generate embeddings for the fetched observations using the `inference::embed` WIT call.
  - Implement a simple $O(N^2)$ cosine similarity check to group observations that have a similarity score > 0.85.

- [x] **Task 4: Implement The Judge Prompt**
  - For each group with $N > 1$ observations, call `inference::generate` with the specialized "Memory Judge" prompt.
  - Parse the JSON response (`has_contradiction`, `golden_fact`).

- [x] **Task 5: Execute Memory Compaction**
  - If the Judge returns a `golden_fact`, format it as a new `McpObservation`.
  - Call `table.atomic-swap` passing the old observation timestamps in `deletes` and the new `golden_fact` in `inserts`.