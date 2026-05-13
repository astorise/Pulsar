# Design: Map-Reduce Architecture



## 1. The Map Phase (Delegation)
When the FaaS gateway receives a prompt, it routes it to the `supervisor`. 
The supervisor invokes the Tier 2 model to generate a JSON array of `SubTask`.
For each `SubTask`, the supervisor uses the Tachyon host API to asynchronously spawn a new `faas/orchestrator` instance. Each instance is assigned a unique `session_id` mapping to a dedicated Git worktree.

## 2. Swarm Execution & Monitoring
The orchestrators work in parallel. Thanks to the Phase 10 MCP Memory (`mcp_observations` table), if Agent A figures out how to handle a new lifetime error, Agent B instantly reads that observation and avoids the same trap.
The supervisor monitors the execution state of all spawned FaaS via a lightweight polling loop or callback.

## 3. The Reduce Phase (Merge & Resolution)
As agents report `Success`, the supervisor triggers a `git merge` of their worktrees.
If a merge conflict occurs, the supervisor does not fail. It reads the conflict markers (`<<<<<<< HEAD`), creates a localized context, and uses a `resolve_conflict` tool to logically merge the code.

## 4. Token Economics (The Circuit Breaker)
Drawing from `rtk`'s analytics capabilities, the supervisor tracks the aggregate token usage of the swarm. If the swarm exceeds a predefined budget (e.g., 500k tokens), the supervisor broadcasts a `Halt` signal to all orchestrators and escalates back to the human developer.