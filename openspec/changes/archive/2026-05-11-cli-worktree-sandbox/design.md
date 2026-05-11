# Design: pulsar-cli Worktree Manager

## Architecture Overview
The FaaS Orchestrator remains completely unchanged. It still thinks it is editing the root of the project. The translation happens in the CLI's `webdav.rs` and session lifecycle.

**Lifecycle:**
1. **Startup:** User types `pulsar-cli` or connects via VS Code.
2. **Sandbox Creation:** The CLI executes `git worktree add -b pulsar/sess-<id> .pulsar/worktrees/sess-<id> HEAD`.
3. **Routing:** The `axum` WebDAV server uses `.pulsar/worktrees/sess-<id>` as its root.
4. **Execution:** The AI explores and modifies the code inside the worktree.
5. **Completion:** When the Orchestrator sends `action_event: finish`, the CLI intercepts it.
6. **Teardown:** The CLI runs `git diff HEAD..pulsar/sess-<id>`. The user is prompted (y/n) to apply the changes. If 'y', it merges and runs `git worktree remove`.