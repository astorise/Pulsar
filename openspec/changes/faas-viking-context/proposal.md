# Proposal: Viking Context FaaS

## Problem Statement
For the Pulsar CLI to autonomously reason over large codebases (e.g., 100k+ lines of code) without exceeding the KV-Cache limits of local GPUs (RTX 3070ti / 2x RTX 3060), it cannot send raw file contents directly to the LLM. We need a semantic compression layer.

## Vision
We introduce `faas/viking-context`, a dedicated WebAssembly component deployed on Tachyon-Mesh. It acts as a smart RAG middleware. When the Pulsar agent needs to explore a repository, it requests context via the `viking://` protocol at three different depths:
- **L0 (Summary):** High-level intent and file purpose (Lowest token cost).
- **L1 (Structure):** Abstract Syntax Tree (AST), signatures, structs, and traits without the implementation bodies (Medium token cost).
- **L2 (Raw):** Exact source code (Highest token cost).

## Value Proposition
This architecture protects the VRAM. The local LLM can scan hundreds of L0/L1 summaries to pinpoint a bug, and only request the L2 payload for the exact file it decides to modify.