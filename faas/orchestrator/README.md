# orchestrator

`orchestrator` coordinates Pulsar sessions. It retrieves workspace context, selects learned skills, applies tiered model escalation, validates tool calls, and records session trace events.

The crate builds as both an `rlib` for native tests and a `cdylib` for WebAssembly component packaging.
