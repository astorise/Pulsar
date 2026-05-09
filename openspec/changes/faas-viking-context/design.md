# Design: faas/viking-context

## Architecture Overview
The FaaS acts as a proxy between the AI Orchestrator (Pulsar CLI) and the file system.

**Execution Flow:**
1. **Input:** The Pulsar agent requests `resolve("viking://src/main.rs", L1)`.
2. **Fetch:** The FaaS uses the `tachyon:mesh/storage-broker` interface to retrieve the raw file bytes.
3. **Semantic Parsing:** - If `L2` is requested, it returns the raw string.
   - If `L1` is requested (and the file is a `.rs` file), it uses a Rust parser (e.g., the `syn` crate) to strip out all `{ ... }` block implementations, leaving only the structural skeleton.
4. **Output:** It packages the result into a `context-response` record and returns it.

## Component Target
- Target: `wasm32-wasip2`.
- Export: `tachyon:ai/viking-context`.