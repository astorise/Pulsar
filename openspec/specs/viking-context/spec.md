# Interface Specification: tachyon:ai/viking-context

This WIT definition standardises how the Pulsar agent fetches hierarchical
codebase memory, and how the component interacts with its Tachyon-Mesh host
dependencies.

```wit
package tachyon:ai@1.0.0;

// ── Host-provided imports ─────────────────────────────────────────────────────

// Will migrate to tachyon:mesh once the mesh WIT is published.
interface storage-broker {
    record file-stat {
        size-bytes: u64,
        /// Unix timestamp (seconds) — used as the cache-invalidation signal.
        modified-secs: u64,
    }

    /// Read raw bytes from a repo-relative or absolute file path.
    read-file: func(path: string) -> result<list<u8>, string>;

    /// Return lightweight metadata without reading the full file.
    stat-file: func(path: string) -> result<file-stat, string>;
}

/// Persistent key-value partition managed by the Tachyon-Mesh host.
/// Configured as disk-backed or memory-only in the deployment manifest.
interface kv-partition {
    get:    func(key: string) -> result<option<list<u8>>, string>;
    set:    func(key: string, value: list<u8>) -> result<_, string>;
    delete: func(key: string) -> result<_, string>;
}

// ── Component export ──────────────────────────────────────────────────────────

interface viking-context {
    /// Depth of the semantic context requested by the agent.
    enum context-level {
        l0-summary,    // High-level summary — lowest token cost
        l1-structure,  // AST skeleton (signatures only) — medium token cost
        l2-raw,        // Exact source code — highest token cost
    }

    /// A resolved chunk of context ready for LLM prompt injection.
    record context-response {
        uri:            string,
        /// Actual level resolved (may differ from request on non-Rust L1 fallback).
        level:          context-level,
        payload:        string,
        /// Rough estimate: len(payload) / 4. Helps the agent manage its budget.
        token-estimate: u32,
    }

    /// Resolve a viking:// URI into a structured, token-efficient payload.
    ///
    /// Cache behaviour:
    ///   - L0 and L1 payloads are looked up in the kv-partition by a key of
    ///     the form `v1:{path}:{level}:{mtime_secs}`.
    ///   - L2 bypasses the cache (raw content is already page-cached by the OS).
    ///   - Cache misses trigger a read + semantic transform + best-effort write.
    resolve: func(uri: string, level: context-level) -> result<context-response, string>;
}

world viking-context-world {
    import storage-broker;
    import kv-partition;
    export viking-context;
}
```

## URI Scheme

```
viking://<repo-relative-path>
```

Examples:
- `viking://src/main.rs`
- `viking://faas/viking-context/src/lib.rs`

## Error Conditions

| Condition | Behaviour |
|-----------|-----------|
| Missing `viking://` prefix | `Err("invalid viking URI: …")` |
| File not found (stat or read) | `Err` propagated from storage-broker |
| File is not valid UTF-8 | `Err("'<path>' is not valid UTF-8")` |
| kv-partition write failure | Silently discarded; result still returned |
| syn parse failure on L1 | Falls back to L2 raw; `resolved_level = l2-raw` |
| L1 requested for non-Rust file | Falls back to L2 raw; `resolved_level = l2-raw` |

## Deployment Manifest Excerpt

```yaml
# manifests/tier1-local.yaml (indicative)
component: viking-context
role: user
kv-partition:
  name: viking-cache
  persistence: disk-backed   # or memory-only for volatile-only cache
```
