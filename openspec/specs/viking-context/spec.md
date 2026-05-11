## Purpose
`tachyon:ai/viking-context` provides hierarchical codebase memory to the Pulsar agent. It resolves `viking://` URIs into token-efficient context payloads while delegating file access and cache persistence to Tachyon-Mesh host services.

## Requirements

### Requirement: Viking Context WIT Contract
The system SHALL expose a `viking-context` interface with context levels `l0-summary`, `l1-structure`, and `l2-raw`, returning a `context-response` with URI, resolved level, payload, and token estimate.

#### Scenario: Agent resolves a Viking URI
- **GIVEN** the agent requests `viking://faas/viking-context/src/lib.rs`
- **WHEN** it calls `resolve` with a context level
- **THEN** the component returns a context response for that URI with a payload and token estimate

### Requirement: Host-Backed Storage Access
The component SHALL import host-provided `storage-broker` functions for `read-file` and `stat-file`, and a `kv-partition` interface for `get`, `set`, and `delete`.

#### Scenario: Component needs source content
- **GIVEN** a valid repo-relative or absolute file path
- **WHEN** the component needs file contents or metadata
- **THEN** it requests them through `storage-broker` instead of touching host storage directly

### Requirement: URI Validation
The component SHALL accept only URIs with the `viking://` scheme and SHALL return an error for any other scheme.

#### Scenario: URI is malformed
- **GIVEN** a URI without the `viking://` prefix
- **WHEN** the agent calls `resolve`
- **THEN** the component returns an invalid URI error

### Requirement: L2 Raw Context
The component SHALL return the exact UTF-8 file contents for `l2-raw` requests and bypass the semantic cache.

#### Scenario: Raw context is requested
- **GIVEN** a valid `viking://` URI for a UTF-8 file
- **WHEN** the agent requests `l2-raw`
- **THEN** the response payload contains the raw file content and the resolved level is `l2-raw`

### Requirement: L0 Summary Context
The component SHALL produce a compact summary for `l0-summary` requests, including file path, line count, and public Rust items when the source parses as Rust.

#### Scenario: Rust file summary is requested
- **GIVEN** a Rust source file with public functions, structs, enums, or traits
- **WHEN** the agent requests `l0-summary`
- **THEN** the response includes a concise summary listing those public items

### Requirement: L1 Structure Context
The component SHALL produce an AST skeleton for Rust files requested at `l1-structure`, preserving declarations while omitting function bodies.

#### Scenario: Rust structure is requested
- **GIVEN** a parseable Rust source file
- **WHEN** the agent requests `l1-structure`
- **THEN** the response includes signatures and type declarations without implementation bodies

### Requirement: Non-Rust Or Parse Fallback
The component SHALL fall back to raw content and set the resolved level to `l2-raw` when `l1-structure` is requested for non-Rust files or Rust parsing fails.

#### Scenario: Non-Rust file is requested at L1
- **GIVEN** a non-Rust UTF-8 file
- **WHEN** the agent requests `l1-structure`
- **THEN** the response returns raw content and reports `l2-raw` as the resolved level

### Requirement: Cache Keyed By File Mtime
The component SHALL cache L0 and L1 payloads in `kv-partition` using a key containing version, path, level, and file modification timestamp.

#### Scenario: Cached summary is fresh
- **GIVEN** a cached payload exists for the requested path, level, and modification timestamp
- **WHEN** the agent requests that context
- **THEN** the component returns the cached payload without recomputing it

### Requirement: Best-Effort Cache Writes
The component SHALL treat cache writes as best effort and SHALL still return a computed response when `kv-partition.set` fails.

#### Scenario: Cache write fails
- **GIVEN** the component computed an L0 or L1 payload
- **WHEN** writing the cache entry fails
- **THEN** the component returns the computed context response anyway
