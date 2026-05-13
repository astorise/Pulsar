# design.md

## Embedded Knowledge Graph Architecture

The `viking-context` FaaS acts as an AST Ingester. Whenever a `.rs` source file is analyzed, it generates a set of semantic triplets (subject, predicate, object) and commits them to the host graph store.

### 1. Data Model (Ontology)
Each code entity uses strict Uniform Resource Names (URNs).
- **Nodes (Subjects/Objects):** - `file:<path>`
  - `module:<path>`
  - `struct:<module_path>::<Name>`
  - `trait:<module_path>::<Name>`
- **Edges (Predicates):** `contains`, `imports`, `implements`, `references`.

### 2. WIT Contract Integration
The component will import the `tachyon:mesh/graph` interface. According to the updated schema:

```wit
package tachyon:mesh@1.0.0;

interface graph {
    record edge {
        subject: string,
        predicate: string,
        object: string,
        properties: string, // Serialized JSON properties
    }

    resource workspace-graph {
        constructor(name: string);
        add-edges: func(edges: list<edge>) -> result<_, string>;
        delete-edges: func(edges: list<edge>) -> result<_, string>;
        traverse: func(subject: string, predicate: string, depth: u32) -> result<list<string>, string>;
    }
}
```

### 3. Guest Rust Implementation & Batching
During the file analysis loop, edges are kept in a local `Vec<Edge>`. Once parsing is complete, the Guest Wasm code initializes the graph resource and commits all edges atomically:

```rust
use bindings::tachyon::mesh::graph::{WorkspaceGraph, Edge};

// Initialize the resource via the WIT constructor
let graph = WorkspaceGraph::new("viking_graph");

// Atomically insert the accumulated edges
if let Err(e) = graph.add_edges(&edges_buffer) {
    eprintln!("Failed to commit AST graph edges: {}", e);
}
```