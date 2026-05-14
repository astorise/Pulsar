# Tasks: Learning Loop and Escalation

- [x] Add the `supervisor` interface dependency to `faas/orchestrator/Cargo.toml`.
- [x] Implement the `supervisor::select_skill` call in the orchestrator's main planning loop.
- [x] Refactor inference calls in the orchestrator to accept and pass the optional `lora_adapter` ID.
- [x] Create the `escalating_inference` retry loop in `faas/orchestrator/src/lib.rs`.
- [x] Define the Tier 1, Tier 2, and Tier 3 model aliases in the orchestrator's state/configuration.
- [x] Add regex/string parsing to detect the `<rabbit_hole_detected>` marker in model outputs to trigger an escalation.
- [x] Add tracing events (`tracing::info!`) whenever an escalation occurs to ensure observability on the Tachyon host.
- [x] Write integration tests mocking the Tachyon host returning a `ResourceExhausted` error to verify the fallback to Tier 2 works correctly.
