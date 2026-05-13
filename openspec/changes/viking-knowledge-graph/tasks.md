# Implementation Tasks

- [ ] **Task 1: Update Viking WIT Contract**
  - Modify `faas/viking-context/wit/world.wit` to add the `graph-query` function and import the new `kv-partition` with the `table` resource.

- [ ] **Task 2: AST Relational Extraction**
  - In `faas/viking-context/src/lib.rs`, update the `extract_rust_skeleton` logic.
  - Detect `syn::Item::Use` to capture dependencies and `syn::Item::Struct/Trait/Enum` to capture defined entities.
  - Return these as a `Vec<(String, Vec<u8>)>` ready for batch insertion.

- [ ] **Task 3: Execute Batch Injection**
  - Instantiate the resource: `let graph_table = Table::new("viking_graph");`.
  - In the main indexing loop, pass the accumulated `Vec` to `graph_table.batch_set(&entries)`.

- [ ] **Task 4: Implement Graph Query Tool**
  - Implement `graph_query(entity: &str)` in `viking-context`. It reads the JSON array from the `graph_table`.
  - In `faas/orchestrator`, add the `QueryGraph { entity: String }` tool, map it to the WIT call, and update the Orchestrator prompt to encourage its use during refactoring.