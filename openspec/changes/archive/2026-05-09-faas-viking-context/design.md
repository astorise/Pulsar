# Design: faas/viking-context

## Architecture Overview

The FaaS acts as a smart proxy between the Pulsar CLI orchestrator and
the local file system. It computes and caches token-efficient representations
of source files at three semantic depths, protecting VRAM from raw-source
flooding.

```
Pulsar CLI
    │  resolve("viking://src/lib.rs", L1)
    ▼
viking-context (WASM / wasm32-wasip2)
    │
    ├─ 1. strip_viking_prefix  →  "src/lib.rs"
    │
    ├─ 2. L2? ──yes──► storage-broker::read_file  →  return raw
    │
    ├─ 3. storage-broker::stat_file  →  mtime
    │        cache_key = "v1:{path}:{level}:{mtime}"
    │
    ├─ 4. kv-partition::get(cache_key)
    │        hit  ──────────────────────────────────► return cached payload
    │        miss ─┐
    │              ▼
    ├─ 5. storage-broker::read_file  →  raw bytes
    │
    ├─ 6. Semantic transform
    │        L0 → build_summary    (syn: public item list + line count)
    │        L1 → extract_rust_skeleton  (syn: signatures, no bodies)
    │
    ├─ 7. kv-partition::set(cache_key, payload)  [best-effort]
    │
    └─ 8. return ContextResponse
```

## Persistence Strategy

viking-context requires a persistent KV cache to avoid recomputing AST
skeletons on every agent request.

**Why not an embedded DB (redb/SQLite)?**
Tachyon's integrity manifest rejects RW volume mounts for user-role FaaS
components, and all user-FaaS writes are routed through an async IPC broker
that is incompatible with the synchronous, transactional file access that
embedded B-tree databases require. mmap is also absent from WASI preview 2.

**Chosen approach — Tachyon kv-partitions**
The deployment manifest declares a `kv-partition` with `disk-backed`
persistence. The host broker handles all concurrency and I/O queuing.
The component only stores and retrieves opaque byte payloads; all cache
logic (key design, invalidation, schema) remains inside viking-context.

## Cache Key Design

```
v1:{file_path}:{level_tag}:{mtime_secs}
```

- `v1:` prefix — schema version; changing it naturally evicts all old entries
  without requiring an explicit flush.
- `mtime_secs` — Unix timestamp from `stat-file`; a file change produces a
  new key, and the stale entry expires passively.
- L2 (raw) is never cached — it equals the file content, which the host OS
  already caches at the page level.

## Cache Write Policy

Cache writes are **best-effort**: a `kv-partition::set` failure is silently
discarded. The result is returned from fresh computation; the next call will
try to cache again. This prevents a write-path failure from surfacing as a
resolution error to the agent.

## Component Target

- Target: `wasm32-wasip2`
- Imports: `tachyon:ai/storage-broker`, `tachyon:ai/kv-partition`
- Export: `tachyon:ai/viking-context`

## Supported Languages for L1

| Language | L1 support | Fallback |
|----------|-----------|---------|
| Rust (`.rs`) | Full AST skeleton via `syn` | — |
| Other | Returns L2 raw | noted in `resolved_level` |

Additional language parsers (TypeScript, Python…) are out of scope for
this change and will be addressed in future changes.
