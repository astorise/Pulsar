# Proposal: MCP Swarm Memory (Paginated)

## Problem Statement
Parallel agents in ephemeral worktrees suffer from context isolation. If Agent A figures out a workaround, Agent B remains ignorant. Sharing data requires a central database, but fetching temporal history (e.g., "the last 15 minutes") via WebAssembly can cause Out-Of-Memory (OOM) crashes if the payload is too large.

## Vision
We are upgrading the `faas/orchestrator` to act as a Swarm. Agents will broadcast standard MCP Observations to a shared RedDB table (`mcp_observations`) keyed by timestamps. To safely read this shared memory, agents will use a strictly paginated `get-range` query loop, protecting the Wasm linear memory.

## Value Proposition
- **Swarm Synergy:** Real-time collaboration between parallel workers.
- **OOM Protection:** The Wasm component model limits data transfer sizes via `limit` and `offset` pagination.
- **Logarithmic Retrieval:** Fetching recent observations is extremely fast thanks to RedDB B-Trees.