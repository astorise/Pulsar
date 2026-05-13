# proposal.md

## Title: Viking Knowledge Graph (CodeWiki) via Tachyon Native Triple Store

### Problem Statement
Currently, the `faas/viking-context` component relies on basic key-value storage and simple keyword matching to parse and store Rust AST skeletons. While fast, it completely lacks systemic and semantic understanding. If an agent modifies a core trait or struct, it cannot preemptively know which other files depend on it, leading to an inefficient trial-and-error loop driven entirely by compiler errors.

### Vision
Inspired by CodeWiki, we will upgrade `faas/viking-context` to extract relational data natively. During the AST extraction pass, the Rust `syn` parser will extract relationship triplets (e.g., `File A` -> `imports` -> `Struct B`). Instead of a KV store, this lightweight semantic graph will be pushed to the new Tachyon native Triple Store (Hexastore over Redb) via the `tachyon:mesh/graph` WIT interface.

### Value Proposition
1. **Preemptive Refactoring:** Agents can use a new `query_graph` tool to ask "Which files depend on `SessionConfig`?" and orchestrate batched updates across the codebase.
2. **Zero-Overhead Insertion:** By utilizing the new Wasm Component Model resource `workspace-graph` and batch functions, thousands of graph edges can be injected into the native database in a single atomic I/O transaction.
3. **Graph-Augmented RAG:** Lays the foundation for navigating codebases semantically up to N-depth dependencies without loading entire workspaces in Wasm memory.