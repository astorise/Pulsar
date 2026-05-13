# Specification: Swarm Supervisor Protocol

## Requirement: Task Decomposition
The `faas/supervisor` SHALL analyze high-level user directives and partition them into non-overlapping sub-tasks (e.g., "Update UI", "Update Database Schema", "Write Tests") to minimize merge conflicts.

## Requirement: Distributed Orchestration
The Supervisor SHALL invoke multiple `faas/orchestrator` components asynchronously. It MUST track their lifecycle states (Running, Failed, Succeeded).

## Requirement: Conflict Resolution
During the Reduce phase, if the Worktree Bridge returns a merge conflict, the Supervisor SHALL intercept the file, parse the Git conflict markers, and generate a semantic resolution before finalizing the commit.

## WIT Interface Updates
The `tachyon:ai/workspace-bridge` MUST support merge operations:
```wit
package tachyon:ai@1.0.0;

interface workspace-bridge {
    // ... existing functions ...

    /// Tries to merge a worktree branch into the main branch
    /// Returns a list of files with conflicts, or an empty list if successful.
    merge-worktree: func(worktree-name: string) -> result<list<string>, string>;
}
```