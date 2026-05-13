# Specification: Context Alignment Protocol

## Requirement: Mandatory Planning
The Orchestrator SHALL reject any attempt by the LLM to use file-system or command-execution tools if the `submit_plan` tool has not been successfully invoked and approved in the current session.

## Requirement: Zero-Cost Wait
The Handshake MUST utilize the `v1:sessions:suspended:{session_id}` KV-store serialization mechanism to ensure the Tachyon host frees all compute resources while waiting for the developer's approval.

## Requirement: Correction Loop
The FaaS MUST support receiving human text input that overrides or refines the proposed plan, enforcing a dialectical loop until consensus is reached.