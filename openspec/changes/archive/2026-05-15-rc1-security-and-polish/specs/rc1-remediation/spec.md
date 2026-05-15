# rc1-remediation Specification

## ADDED Requirements

### Requirement: Structured Command Boundary
The orchestrator WIT boundary SHALL pass commands as an executable plus argument list and receive an explicit command result containing exit code, stdout, and stderr.

#### Scenario: Command result is evaluated
- **GIVEN** a host command returns an exit code
- **WHEN** the orchestrator records the command result
- **THEN** command success is determined from `exit-code == 0`

#### Scenario: Shell flattening is avoided
- **GIVEN** the orchestrator sends a command to the workspace bridge
- **WHEN** the WIT call is made
- **THEN** the executable and argument list are passed as separate values

### Requirement: Component-Normalized Workspace Paths
The orchestrator SHALL normalize user-provided paths with `std::path::Component` before host file, context, or LSP calls.

#### Scenario: Parent traversal above workspace is blocked
- **GIVEN** a path contains parent-directory components that escape the workspace
- **WHEN** the path guard validates it
- **THEN** the operation is rejected before a host call is emitted

#### Scenario: Internal parent traversal is normalized
- **GIVEN** a path contains parent-directory components that remain within the workspace
- **WHEN** the path guard validates it
- **THEN** the resulting path is normalized before use

### Requirement: Git Command Sub-Allowlist
The command sandbox SHALL restrict `git` invocations to safe verbs and reject configuration override arguments.

#### Scenario: Git configuration override is rejected
- **GIVEN** a command starts with `git`
- **WHEN** any argument starts with `-c`, `-C`, or `--exec-path`
- **THEN** the command is rejected as forbidden

#### Scenario: Safe Git verb is allowed
- **GIVEN** a command starts with `git`
- **WHEN** the first argument is an allowed verb such as `status`
- **THEN** the command may proceed if all other sandbox checks pass

### Requirement: Wasmtime Component E2E Harness
The E2E harness SHALL load an actual orchestrator WebAssembly artifact through `wasmtime::component::Component::from_file`.

#### Scenario: Component artifact loads
- **GIVEN** the orchestrator has been built for `wasm32-wasip1`
- **WHEN** the E2E runner converts the artifact to a component and loads it from disk
- **THEN** Wasmtime accepts the component without falling back to native Rust module tests

### Requirement: RC1 Repository Hygiene
The repository SHALL remove obsolete host/WIT duplicates and document the security threat model.

#### Scenario: Orphaned host code is removed
- **GIVEN** Tachyon-Mesh owns host-side implementations
- **WHEN** RC1 cleanup is applied
- **THEN** the orphaned `core-host` source files are removed from the active repository

#### Scenario: Threat model is documented
- **GIVEN** a contributor reviews repository security
- **WHEN** they open `SECURITY.md`
- **THEN** they can distinguish LLM, orchestrator, host, and workspace trust boundaries
