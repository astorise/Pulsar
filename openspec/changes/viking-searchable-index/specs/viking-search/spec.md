# Interface Update: tachyon:ai/viking-context

We expand the WIT contract to support semantic search.

```wit
package tachyon:ai@1.0.0;

// ... existing imports ...

interface viking-context {
    // ... existing enums and records ...

    /// Resolve a specific URI (Existing)
    resolve: func(uri: string, level: context-level) -> result<context-response, string>;

    /// NEW: Search the L0 cached summaries for keywords
    /// Returns a list of matching viking:// URIs
    search: func(query: string) -> result<list<string>, string>;
}
```

## Orchestrator Tooling Update
The `faas/orchestrator` must add the following to its `ToolCall` enum:
```rust
SearchVikingContext { query: String }
```