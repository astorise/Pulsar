# Proposal: Skill Retrieval Loop

## Problem Statement
The `skill-extractor` successfully distills complex traces into reusable `SKILL.md` files located in the `.pulsar/skills/` directory. However, the `faas/orchestrator` does not load these files during the `submit_input` phase. This breaks the Hermès paradigm: the agent fails to compound its intelligence and will waste VRAM and Tier 2 compute repeating past mistakes.

## Vision
We will introduce a `Recalling` phase to the Orchestrator's state machine. Before executing the first `Think` step for a new user prompt, the Orchestrator will query the WebDAV bridge for available skills in `.pulsar/skills/`. Relevant skills will be injected directly into the system prompt as "Past Experiences".

## Value Proposition
- **O(1) Resolution:** Tasks that previously took 5 tool calls (escalating to Tier 2) can now be solved in 1 tool call by the local Tier 1 (7B model) because the exact workflow is provided in the prompt.
- **True Compounding AI:** The more the developer uses Pulsar, the faster and cheaper it becomes on that specific codebase.