# tasks.md

## Implementation Tasks (Phase 9)

- [x] **Task 1: Update Wasm World Bindings**
  - Modify `faas/viking-context/wit/world.wit` to import `tachyon:mesh/graph`.
  - Regenerate the Rust bindings using `wit-bindgen`.

- [x] **Task 2: Refactor the AST Parser**
  - Remove the legacy `kv-partition` key-value storage logic from `faas/viking-context/src/lib.rs` for AST mapping.
  - Implement `syn::visit::Visit` to extract `Use`, `ItemStruct`, `ItemTrait`, and `ItemImpl` items.

- [x] **Task 3: URN and Triplet Generation**
  - Create a deterministic naming module to generate strict URNs (e.g., `struct:path::to::mod::MyStruct`).
  - Serialize AST metadata (line numbers, column, visibility) into the `properties` JSON string field.

- [x] **Task 4: Implement Batch I/O**
  - Add an edge buffer (`Vec<Edge>`) to the visitor state.
  - Implement the resource instantiation `WorkspaceGraph::new("viking_graph")`.
  - Trigger `add_edges` safely, ensuring `Result` matching is implemented to handle Host-side I/O errors.

- [x] **Task 5: Synchronization & Diffing**
  - Implement the `delete_edges` logic when a file update event is captured.
  - Use `traverse` to fetch the prior AST state of a given file to perform the cleanup delta.

- [x] **Task 6: Orchestrator Tool Integration**
  - Expose the new `QueryGraph { entity: String, depth: u32 }` capability to the Pulsar Orchestrator.
  - Write integration tests mocking a Rust file change and verifying the `add-edges` calls on the Host.