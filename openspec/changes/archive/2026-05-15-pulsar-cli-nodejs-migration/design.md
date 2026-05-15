# Design: Node.js Pulsar CLI Bridge

## Architecture Components

1. **The WebDAV Bridge (`src/webdav.ts`)**
   * Replace the Rust `webdav::spawn` with a Node.js implementation using `webdav-server` v2.
   * Dynamically generate the authorization token and expose the active git worktree (or sandbox) as an in-memory or securely scoped file system.

2. **The Orchestrator WS Client (`src/ws-client.ts`)**
   * Use the standard `ws` Node.js library to maintain the connection with the Tachyon Host.
   * Send the initial `ClientMessage::Init` payload containing the `workspace_url` and `workspace_token`.

3. **Git Sandbox & Merge (`src/git.ts`)**
   * Replace the Rust Git integration with `simple-git` or native `child_process` wrappers.
   * Replicate the `Sandbox::create`, `diff_stat`, and `apply_patch_to_repo` logic to safely stage agent changes.

4. **Human Bridge / REPL (`src/repl.ts`)**
   * Implement the "Rabbit Hole" and Context Handshake logic using standard Node.js `readline` or `inquirer` for interactive terminal prompts (`[y/N]`).

## Dependency Graph
* `webdav-server` (WebDAV protocol)
* `ws` (WebSocket client)
* `simple-git` (Git operations)
* `chalk` & `inquirer` (Terminal UI and Human Bridge interaction)
* `typescript` & `esbuild` (Build pipeline)