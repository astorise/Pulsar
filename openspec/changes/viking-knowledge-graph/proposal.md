# Proposal: Viking Knowledge Graph (CodeWiki)

## Problem Statement
The current `search_viking_context` tool relies on simple keyword matching. While fast, it lacks systemic understanding. If the agent modifies a core trait or struct, it cannot preemptively know which other files depend on it, leading to a trial-and-error loop driven by compiler errors.

## Vision
Inspired by CodeWiki, we will upgrade `faas/viking-context` to extract relational data. During the L1 (AST Skeleton) extraction pass, the Rust `syn` parser will also extract relationship triplets (e.g., `File A` -> `imports` -> `Struct B`). This creates a lightweight semantic graph stored in the Tachyon RedDB backend. 

## Value Proposition
- **Preemptive Refactoring:** The agent can use a new `query_graph` tool to ask "Which files depend on `SessionConfig`?" and update them all in a single batch.
- **Zero-Overhead Insertion:** By utilizing the new Wasm Component Model `resource table` and the `batch-set` function, thousands of graph edges can be injected into the database in a single I/O transaction.