# Proposal: Swarm Supervisor

## Problem Statement
Currently, a user assigns a task to a single `faas/orchestrator`. If the task is massive (e.g., "Migrate the entire codebase to Axum 0.7"), the agent gets overwhelmed, the context window explodes, and the sequential execution takes too long. Even though we have isolated worktrees and shared MCP memory, we lack a "Tech Lead" to coordinate parallel execution.

## Vision
We introduce a new component: `faas/supervisor`. This FaaS acts as the router and merger of the Swarm using a Map-Reduce paradigm. 
- **Map:** It breaks a complex user prompt into independent sub-tasks and spawns multiple `faas/orchestrator` instances in parallel, each in its own worktree.
- **Reduce:** Once the sub-agents complete their tasks, the supervisor reviews the code, resolves git conflicts, and merges everything into the main branch.

## Value Proposition
- **Extreme Speed:** A 3-hour refactoring task can be completed in 15 minutes by spawning 12 agents simultaneously on the Tachyon cluster.
- **Cost Control:** The supervisor enforces a "Token Budget" across the swarm (inspired by RTK's economic analytics) to prevent runaway loops.