# Specification: Semantic Graph Protocol

## Requirement: Relational Edge Extraction
The `viking-context` FaaS SHALL extract type definitions and `use` imports during the `syn` parsing phase of L1 context generation without requiring a separate file read.

## Requirement: Resource-Based KV Interaction
The FaaS MUST use the `tachyon:ai/kv-partition` WIT interface using the `resource table` syntax to prevent handle leaks. It SHALL use `batch-set` for all insertions to minimize RedDB auto-commit overhead.

## Requirement: Graph Query Interface
The `tachyon:ai/viking-context` WIT interface SHALL expose a `graph-query` function.

```wit
package tachyon:ai@1.0.0;

interface viking-context {
    // ... existing functions (resolve, search) ...

    /// Queries the knowledge graph for a specific entity (struct, trait, fn).
    /// Returns a list of viking:// URIs that depend on or import this entity.
    graph-query: func(entity-name: string) -> result<list<string>, string>;
}
```