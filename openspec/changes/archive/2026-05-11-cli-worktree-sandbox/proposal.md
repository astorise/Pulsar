# Proposal: CLI Git Worktree Sandbox

## Problem Statement
Currently, the `pulsar-cli` WebDAV server exposes the developer's live current working directory (`cwd`). When the remote Orchestrator FaaS calls the `edit_file` tool, it overwrites the user's active files. This prevents true asynchronous parallel execution (the "Night-Shift" mode) and makes AI mistakes dangerous to the local codebase.

## Vision
We will implement an invisible sandboxing mechanism inside the `pulsar-cli` using `git worktree`. 
When a new agent session starts, the CLI will automatically create a hidden, ephemeral git worktree (e.g., `.pulsar/worktrees/sess-xyz`). The WebDAV server will be rooted in this isolated folder instead of the `cwd`.

## Value Proposition
- **Zero Risk:** The AI can delete files, write bad code, or break tests without ever affecting the developer's active editor.
- **Parallelism:** We can run 5 different `pulsar-cli` instances tackling 5 different features simultaneously on the same repository.
- **Review Before Merge:** When the FaaS calls `finish`, the CLI will prompt the user to review the diff. If approved, the CLI will squash-merge the worktree back into the main branch and delete the sandbox.