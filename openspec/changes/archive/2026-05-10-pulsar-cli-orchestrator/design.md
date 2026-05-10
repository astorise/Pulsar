# Design: pulsar-cli

## Architecture
The CLI uses an Actor-based architecture to handle concurrent AI loops and terminal UI without blocking.

- **REPL Actor:** Manages the terminal interface (rustyline or standard stdin/stdout).
- **Orchestrator Actor:** Runs the `Think -> Act -> Observe` agentic loop.
- **Tachyon Client (gRPC):** Uses `tonic` and `prost` to stream bidirectional contexts to the Tachyon Kubernetes cluster (k3s/Talos).

## Git Worktree Parallelism (Foundation)
To support parallel reasoning (e.g., researching a bug while running tests), the CLI will use a pattern inspired by `worktrunk` and `kodo`. We will scaffold a `workspace` module that can spawn isolated `git worktree` instances.