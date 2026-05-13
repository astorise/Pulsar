# Implementation Tasks

- [x] **Task 1: Scaffold FaaS Supervisor**
  - Create the `faas/supervisor` project.
  - Define its core prompt: *"You are the Tech Lead. Do not write code. Break down the user's request into independent files/components to be worked on in parallel."*

- [x] **Task 2: Implement the Map Function**
  - In `faas/supervisor`, implement the logic to parse the sub-tasks JSON.
  - For each task, call the cluster's execution engine to spawn a `faas/orchestrator`.

- [x] **Task 3: Worktree Merge Tooling**
  - Update `faas/orchestrator/wit/world.wit` and `pulsar-cli` to implement `merge-worktree`.
  - The CLI should execute `git merge <worktree_branch> --no-commit` and parse `git diff --name-only --diff-filter=U` to return conflicts.

- [x] **Task 4: Implement the Reduce Function**
  - In `faas/supervisor`, implement a polling mechanism to wait for all child orchestrators to finish.
  - Sequentially call `merge-worktree` for each branch.
  - If conflicts are returned, read the file, extract the conflict markers, and ask the LLM to provide the unified code.

- [x] **Task 5: Token Budget Circuit Breaker**
  - Implement a shared counter in the `kv-partition` (`v1:metrics:swarm_tokens:{session}`).
  - Each orchestrator must increment this counter after every inference call.
  - If the counter exceeds the user's configured limit, the FaaS panics gracefully, halting the swarm.