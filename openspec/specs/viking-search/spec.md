# viking-search Specification

## Purpose
TBD - created by archiving change viking-searchable-index. Update Purpose after archive.
## Requirements
### Requirement: Viking Search Interface
The `viking-context` interface SHALL expose `search(query: string)` and return matching `viking://` URIs from cached L0 summaries.

#### Scenario: Search cached summaries
- **GIVEN** L0 summaries exist in the Viking cache
- **WHEN** the orchestrator calls `search` with a keyword query
- **THEN** Viking returns the URIs whose summaries match the query case-insensitively

### Requirement: Cache Key Listing
The `kv-partition` interface SHALL expose `list-keys` so Viking can discover cached L0 summary keys without scanning files directly.

#### Scenario: Viking enumerates L0 cache entries
- **GIVEN** the cache contains keys for multiple context levels
- **WHEN** Viking searches for summaries
- **THEN** it only reads keys associated with the L0 level

### Requirement: Search-First Orchestration
The orchestrator SHALL provide a `search_viking_context` tool and instruct the model to search before reading or modifying files.

#### Scenario: Agent needs relevant files
- **GIVEN** a user asks for a code change without naming every file
- **WHEN** the model chooses the next tool
- **THEN** the prompt offers `search_viking_context` before lower-level file operations
