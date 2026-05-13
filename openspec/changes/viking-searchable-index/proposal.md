# Proposal: Viking Searchable Index

## Problem Statement
Currently, `faas/viking-context` uses a "Lazy Caching" strategy. It only computes L0/L1 semantic contexts when the agent explicitly requests a specific `viking://` URI. 
This creates two bottlenecks:
1. The agent suffers high latency on the first file read.
2. The agent must guess or list directories to find relevant files, consuming unnecessary Tier 2 LLM tokens and VRAM.

## Vision
Inspired by Airbyte's "Context Store" architecture, we are upgrading Viking to a proactive, searchable index:
- **Proactive Indexing:** When `faas/orchestrator` starts a session, it will spawn a background crawler over the WebDAV mount to eagerly ask Viking to compute and cache L0 summaries for all codebase files.
- **Searchable Interface:** We add a `search(query)` function to the Viking FaaS. The agent can now use a `search_viking_context` tool to find files matching a concept (e.g., "Where is the authentication middleware?") before requesting the full L1/L2 payload.

## Value Proposition
This strictly separates "Ask" (Exploration) from "Act" (Execution). By searching an index rather than guessing file paths, token usage will drop drastically, and the agent's accuracy in large repositories will skyrocket.