# Specification: Contradiction Resolution Protocol

## Requirement: Atomic GC Transactions
The `tachyon:ai/kv-partition` interface MUST support atomic transactions to ensure that if the GC process crashes while replacing facts, the database is not left in an inconsistent state (where both the old facts and the new golden fact are deleted/present).

## Requirement: Semantic Embeddings
The `tachyon:ai/inference` interface MUST expose an `embed` function to allow the GC to generate vector representations of strings for clustering.

## WIT Interface Updates

```wit
package tachyon:ai@1.0.0;

interface inference {
    // ... existing generate function ...
    
    /// Generates a vector embedding for a given string
    embed: func(model: string, text: string) -> result<list<f32>, string>;
}

interface kv-partition {
    // ... existing resource table ...
    
    /// Executes a batch of deletions and insertions atomically
    atomic-swap: func(deletes: list<string>, inserts: list<tuple<string, list<u8>>>) -> result<_, string>;
}
```