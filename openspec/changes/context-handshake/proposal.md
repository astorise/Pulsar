# Proposal: Context Handshake

## Problem Statement
When given a complex task, LLMs can immediately dive into tool execution with a flawed understanding of the environment or the goal (the "Consequence Drift" anti-pattern). If the agent starts modifying files on the `main` branch instead of a feature branch, or misunderstands the core architectural directive, the swarm becomes destructive.

## Vision
We introduce a "Context Handshake" phase. Upon receiving a new user prompt, the `faas/orchestrator` does not immediately act. Its first iteration is forced to generate a structured summary of the goal, the active Git branch, and a step-by-step plan.
It then suspends its execution and sends this Handshake to the CLI. The human developer must explicitly approve the plan or provide a course correction before the agent is allowed to touch the codebase.

## Value Proposition
- **Total Control:** The developer acts as the gatekeeper. Destructive or hallucinatory plans are killed in seconds.
- **Token Economy:** Prevents the swarm from wasting API budgets on misunderstood tasks.
- **Cognitive Alignment:** Forces the Tier 2 model to synthetically map out its reasoning before writing code, inherently improving its success rate (Chain-of-Thought).