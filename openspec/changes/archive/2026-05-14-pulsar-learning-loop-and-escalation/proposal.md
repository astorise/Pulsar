# Proposal: Wiring the Continuous Learning Loop and Model Escalation

**Date**: 2026-05-16
**Status**: Active
**Category**: AI Orchestration & Architecture

## Context & Motivation
The recent independent audit revealed that two of Pulsar's core architectural promises—"Continuous Learning" and "Tiered Inference"—are currently implemented as disconnected stubs. While the `skill-extractor` generates `SKILL.md` files and registers them, the `orchestrator` never queries the `supervisor` to use these skills. Furthermore, inference calls are hardcoded to a single model, completely bypassing the fallback mechanism to higher-tier models when local execution fails or lacks confidence.

## Proposed Changes
1. **Close the Learning Loop**: Connect the `orchestrator` to the `supervisor`. Before executing a complex task, the orchestrator must query the supervisor (which interfaces with the semantic graph) to retrieve the most relevant `lora_adapter` ID.
2. **LoRA Adapter Injection**: Modify the orchestrator's inference calls to pass the retrieved `lora_adapter` ID to the Tachyon host environment.
3. **Tiered Escalation Routing**: Implement a fallback loop in the orchestrator. If a Tier 1 (Local/Edge) model returns a low-confidence score, triggers a Wasm resource limit, or explicitly yields, the orchestrator must seamlessly re-route the request to a Tier 2 (Cluster) or Tier 3 (Cloud) model.

## Impact
* **Autonomous Improvement**: Agents will finally benefit from previously extracted skills, increasing their success rate on repeated complex tasks.
* **Resilience**: The swarm will survive local GPU constraints or context limits by intelligently escalating to larger models only when strictly necessary.