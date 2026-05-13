# Proposal: Cognitive Garbage Collector (Contradiction Engine)

## Problem Statement
As the Swarm operates, multiple ephemeral agents write observations and facts into the shared `redb` MCP memory. Over time, the codebase evolves, rendering old facts obsolete, or parallel agents deduce conflicting conclusions. This "Memory Slop" poisons the RAG context window, causing future agents to hallucinate or revert correct code based on outdated observations.

## Vision
Inspired by the architecture of `garrytan/gstack`, we will introduce a new background FaaS: `faas/cognitive-gc`. This component acts as the "Sleep/Dream cycle" of the Pulsar system. It continuously scans the `mcp_observations` table using semantic similarity (embeddings) to group related facts. When it detects a logical contradiction (e.g., Fact 1 vs Fact 2), it invokes an LLM Judge to evaluate the facts against the current Git state, synthesize the absolute truth, and execute an atomic RedDB transaction to replace the conflicting entries with a unified "Golden Fact".

## Value Proposition
- **Self-Healing Memory:** The system autonomously curates its own knowledge base, preventing consequence drift without human intervention.
- **Context Window Optimization:** Consolidating 5 repetitive or conflicting observations into 1 Golden Fact saves hundreds of tokens per inference call.
- **Eventual Consistency:** Guarantees that the Swarm's mental model always converges towards the actual state of the codebase.