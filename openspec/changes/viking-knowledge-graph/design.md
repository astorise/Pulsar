# Design: Embedded Knowledge Graph with Batch Injection

## 1. Edge Extraction
When `extract_rust_skeleton` parses a `.rs` file, it will populate a secondary metadata object tracking:
- **Definitions:** Which `struct`, `enum`, `fn`, or `trait` are declared in this file.
- **Dependencies:** Which types are imported via `use` statements.

## 2. Table Storage (Batch-Oriented)
The FaaS uses the Tachyon `kv-partition` interface to open a dedicated graph table.
- **Initialization:** `let graph_table = Table::new("viking_graph");`
- During the L1 analysis of a file, Viking accumulates all discovered edges in a local `Vec<(String, Vec<u8>)>`.
- At the end of the file analysis, it executes a single `graph_table.batch-set(entries)` call to commit the graph to RedDB.

## 3. The Orchestrator Interface
The Orchestrator receives a new tool: `QueryGraph { entity: String }`.
This tool calls the new WIT function `viking-context::graph-query`, returning the list of dependent URIs.