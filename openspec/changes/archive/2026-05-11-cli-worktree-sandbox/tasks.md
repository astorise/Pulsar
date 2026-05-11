# Implementation Tasks

- [x] **Task 1: Add Git abstractions**
  - Create `pulsar-cli/src/git.rs`.
  - Implement helper functions using `std::process::Command` to execute `git worktree add`, `git diff`, `git merge`, and `git worktree remove`.

- [x] **Task 2: Update CLI Initialization**
  - Modify `pulsar-cli/src/main.rs`. Before spawning the WebDAV task, check if the current directory is a git repository.
  - If yes, use the `git` helpers to create a worktree in `.pulsar/worktrees/<token>`.
  - Pass the worktree path to the WebDAV server instead of `env::current_dir()`.

- [x] **Task 3: Ignore Worktrees**
  - Create logic to automatically add `.pulsar/` to the local `.git/info/exclude` so the agent doesn't accidentally commit its own sandboxes.

- [x] **Task 4: Intercept Finish Event**
  - Modify `pulsar-cli/src/ws_client.rs` to detect when a `ServerMessage::ActionEvent` has an action of `"finish"`.
  - Send a signal back to the main thread to initiate the teardown process.

- [x] **Task 5: Implement Teardown & Merge UI**
  - When the finish signal is caught, pause the REPL.
  - Print the diff stats from the worktree.
  - Ask `Apply these changes? [y/N]`.
  - Clean up the worktree and the branch regardless of the choice.
