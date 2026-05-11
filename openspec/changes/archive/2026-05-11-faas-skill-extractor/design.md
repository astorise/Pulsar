# Design: faas/skill-extractor

## Architecture Overview
This component is a stateless `wasm32-wasip2` function. It acts as an asynchronous processor.

**Execution Flow:**
1. **Input:** The Pulsar CLI calls the FaaS, providing the `task_description` and the `execution_trace` (JSON string of the agent's memory).
2. **Analysis:** The FaaS calls the `tachyon:ai/inference` interface (targeting the local 27B model) with a specific meta-prompt: *"Analyze this execution trace. Extract the core lesson and write a concise, deterministic SKILL.md guide."*
3. **Storage:** The FaaS uses `tachyon:mesh/storage-broker` to save the resulting string into `.pulsar/skills/<sanitized_task_name>.md`.
4. **Output:** Returns the URI of the newly created skill.

## Component Target
- Target: `wasm32-wasip2`.
- Export: `tachyon:ai/skill-extractor`.