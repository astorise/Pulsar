# Design: Human Bridge Integration

## Scope Boundary
This repository owns the Pulsar Wasm agents, CLI, and tests. Tachyon `core-host`, `redb` approval persistence, and REST approval endpoints are external to this workspace, so this change implements the Pulsar-side contract and records the external host requirements in the archived task notes.

## Orchestrator Contract
The orchestrator imports a `human-bridge` WIT interface with `approved`, `rejected`, and `modified` outcomes. The import is only used by the Wasm component path; native tests exercise the scoring and modified-command handling through pure Rust helpers.

## Impact Scoring
The scorer keeps read-only tools below the approval threshold and raises mutating operations above it. File edits default to approval-required, with extra weight for workflow, manifest, lockfile, and script changes. Commands are scored by executable and verb so harmless inspection commands stay low impact while operations such as `git push`, `git rebase`, and package publishing require approval.

## Approval Handling
For `approved`, the original operation executes. For `rejected`, the orchestrator records a human rejection observation and skips execution. For `modified`, file edits use the supplied replacement contents, while commands are reparsed and revalidated before execution.
