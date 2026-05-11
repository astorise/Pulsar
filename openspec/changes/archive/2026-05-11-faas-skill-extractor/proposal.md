# Proposal: Skill Extractor FaaS

## Problem Statement
When the Pulsar CLI delegates a complex task to the Tier 2 or Tier 3 models, the agent might take 5 to 10 iterative tool-calls (reading context, trying a fix, getting a compiler error, fixing it) to succeed. If the user asks the exact same type of refactoring a week later, the agent will waste the same amount of time and compute resources.

## Vision
We introduce `faas/skill-extractor`, a WebAssembly background worker deployed on Tachyon-Mesh. 
When the Pulsar CLI successfully finishes a complex task, it asynchronously sends the "Execution Trace" (the history of prompts, tool calls, and results) to this FaaS. 
The FaaS uses the local Tier 2 model (`wasi-nn`) to analyze the trace, deduce the "golden path" (the optimal workflow), and generates a reusable Markdown skill (`SKILL.md`).

## Value Proposition
This creates a compounding intelligence effect. The generated `SKILL.md` files are saved into the repository's `.pulsar/skills/` directory. Future executions by the Tier 1 (local 7B model) will proactively load these skills via the Viking Context, allowing the small model to solve complex problems instantly without escalating to Tier 2.