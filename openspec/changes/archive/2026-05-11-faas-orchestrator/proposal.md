# Proposal: Pulsar Orchestrator FaaS

## Problem Statement
Developing agentic logic within the CLI (Tier 1) creates a "Fat Client" that is hard to update and tightly coupled to local hardware. Furthermore, managing the complex state of an agent (Think-Act-Observe loop) across different environments (Windows, Linux, VS Code) leads to implementation fragmentation.

## Vision
We move the entire "Brain" of Pulsar into a centralized FaaS: `faas/orchestrator`. 
The orchestrator acts as a stateful agent that:
- Connects to the `pulsar-cli` via WebSocket for real-time interaction.
- Accesses the developer's local workspace via a WebDAV mount exposed by the CLI.
- Coordinates calls to `viking-context` for semantic code analysis.
- Triggers the `skill-extractor` to persist learned workflows.

## Value Proposition
- **Thin Client:** The CLI only needs to provide file access (WebDAV) and a terminal/UI (WebSocket).
- **Persistent Reasoning:** The agent can continue a complex task on the server even if the CLI is temporarily disconnected.
- **Unified Logic:** A single point of truth for the agentic loop, regardless of the IDE or OS used by the developer.