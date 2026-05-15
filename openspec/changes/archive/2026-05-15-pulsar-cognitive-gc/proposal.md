# Proposal: Cognitive GC Spec Polish

## Problem
The archived `cognitive-gc` capability left the main specification with a placeholder Purpose. This makes the capability harder to scan in `openspec validate --all --strict` output and leaves the spec looking unfinished even though the runtime and requirements already exist.

## Change
Replace the placeholder Purpose with a concise description of the capability's role in Pulsar memory compaction.

## Impact
- Clarifies the existing `cognitive-gc` capability.
- No runtime behavior changes.
- No requirement semantics change.
