# Security Model

Pulsar treats model output as untrusted input. The LLM may propose file paths, commands, and tool calls, but the host and WebAssembly components must validate every boundary before touching the workspace.

## Trust Boundaries

- LLM: untrusted. It can suggest actions only through structured tool calls.
- Orchestrator component: partially trusted policy layer. It validates paths, command arguments, and session state before asking the host to act.
- Tachyon-Mesh host: trusted runtime boundary. It owns workspace credentials, filesystem access, process execution, KV storage, and network access.
- Workspace: protected data boundary. Files may be read or changed only through validated relative paths inside the workspace root.

## Command Execution

Commands must be represented as an executable plus an argument list. The orchestrator rejects shell operators, null bytes, unknown executables, and unsafe Git configuration overrides such as `-c`, `-C`, and `--exec-path`.

Command success is determined from the host-provided exit code, not by parsing output text.

## Filesystem Access

Paths are normalized with `std::path::Component`. Absolute paths, platform prefixes, null bytes, and parent traversal above the workspace root are rejected before any host call.

## Secret Handling

Workspace tokens must not be stored in static session state. The host or runtime environment provides secrets when authorization is needed, and callers must avoid writing those values into logs, traces, or archived OpenSpec changes.

## Reporting

Report security issues privately to the repository owner before public disclosure. Include reproduction steps, affected boundary, and whether the issue requires LLM-generated input, host compromise, or direct workspace access.
