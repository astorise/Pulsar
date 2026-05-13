# Proposal: Dynamic Skillpack Registry

## Problem Statement
Currently, FaaS agent behaviors (System Prompts, Tools) are hardcoded in Rust or require slow file-system reads at runtime. As the Swarm grows to dozens of specialized agents (Reviewer, UI Tester, DB Migrator), managing them becomes complex, and the Supervisor struggles to know which agent to route a task to without blowing up its context window.

## Vision
Inspired by `gstack`'s `SKILL.md` paradigm, we will allow developers to define new agents purely in Markdown within a `.pulsar/skills/` directory. 
We will introduce a "Skillify" build step. This process will parse the Markdown, extract the instructions and tool schemas, generate a vector embedding of the skill's purpose, and compile it all into a high-speed `redb` table (`pulsar_skill_registry`).
When the Swarm Supervisor receives a task, it uses RAG (Vector Search) against this table to dynamically discover and instantiate the exact right agent for the job in less than a millisecond.

## Value Proposition
- **Extreme DX (Developer Experience):** Creating a new agent is as simple as writing a Markdown file. No Rust compilation required.
- **Semantic Routing:** The Supervisor can dynamically discover new skills it wasn't originally programmed to know about.
- **Zero-Latency:** By pre-processing the Markdown into an embedded KV-Cache, the FaaS workers boot instantly with their context pre-hydrated.