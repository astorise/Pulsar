# Pulsar

Pulsar is a distributed autonomous engineering system built on Tachyon-Mesh. It runs specialized Rust WebAssembly components as isolated FaaS agents, coordinates them through a thin CLI, and keeps shared project context in Viking Context so agents can work with bounded memory and predictable escalation.

## Vision

Pulsar turns language models into a coordinated engineering loop instead of a single chat surface. The system is designed around explicit planning, context-aware execution, automatic escalation, and durable learning from successful work.

Core goals:

- Keep humans in control with clear planning and resumable sessions.
- Run agents as sandboxed WebAssembly components on Tachyon-Mesh.
- Use tiered inference so simple work stays local and hard work escalates.
- Capture reusable skills from successful resolutions.
- Keep context compact through Viking summaries, structure views, and targeted reads.

## Current State

The repository currently contains:

- FaaS crates for orchestration, context retrieval, inference routing, skill extraction, supervision, cognitive cleanup, and browser automation.
- A thin `pulsar-cli` client for starting sessions against a Tachyon-Mesh workspace.
- OpenSpec proposals and archived specs describing the product direction.
- Workspace CI for tests, clippy, and secret scanning.

Pulsar is still early-stage infrastructure. The crates are structured for native unit tests and WebAssembly component builds, while deeper Tachyon-Mesh host integration is implemented incrementally through OpenSpec changes.

## Architecture

```text
pulsar-cli
  |
  v
orchestrator FaaS
  |-- viking-context FaaS
  |-- inference-gateway FaaS
  |-- skill-extractor FaaS
  |-- supervisor FaaS
  |-- cognitive-gc FaaS
  `-- browser-agent FaaS

Tachyon-Mesh provides the component runtime, host APIs, workspace bridge, KV storage, and isolation boundary.
```

## Development

Run the full local check set:

```bash
cargo test --workspace
cargo test --test wasm_e2e_runner
cargo clippy --workspace --all-targets -- -D warnings
```

Validate OpenSpec artifacts:

```bash
openspec validate --all --strict --no-interactive
```

## License

Pulsar is distributed under the MIT License.
