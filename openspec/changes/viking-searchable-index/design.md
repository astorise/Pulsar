# Design: Searchable Index Architecture

## 1. Proactive WebDAV Crawler (`faas/orchestrator`)
When `start_session` is called, the orchestrator triggers an asynchronous `PROPFIND` crawl over the WebDAV workspace. For every `.rs`, `.toml`, or `.md` file found, it fires a non-blocking `resolve(uri, l0-summary)` request to `viking-context` to warm up the Tachyon `kv-partition`.

## 2. Viking Search Implementation (`faas/viking-context`)
Since Tachyon's `kv-partition` is a simple Key-Value store, implementing search requires scanning. 
When `search(query)` is called:
1. Viking retrieves all cached L0 payloads. *(Note: Requires a new `scan` or `list_keys` capability from the storage host, or maintaining an index manifest).*
2. It performs a fast, case-insensitive keyword search (grep-style) across the L0 summaries.
3. It returns a ranked list of `viking://` URIs matching the query.

## 3. Tool Reorganization
The orchestrator's system prompt is updated to strictly enforce the **Connect -> Ask -> Act** pipeline:
- **Ask Tools:** `search_viking_context`, `read_viking_context`
- **Act Tools:** `edit_file`, `run_command`