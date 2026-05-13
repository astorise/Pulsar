# Proposal: Human Action Bridge

## Problem Statement
AI agents are constrained to software boundaries. When a task requires physical interaction (e.g., "Plug in the USB cable", "Read the 2FA code") or qualitative judgment ("Does this animation feel right?"), the agent either hallucinates a success or fails. Furthermore, in a FaaS architecture like Tachyon, blocking a thread to wait for a human wastes compute resources and triggers timeouts.

## Vision
We introduce a `request_human_action` tool. When the Orchestrator encounters a physical or qualitative blocker, it uses this tool. Instead of blocking, the FaaS serializes its current session state to the `kv-partition`, sends an `AwaitingHuman` signal to the `pulsar-cli` via the WebSocket, and gracefully terminates. 
Once the developer completes the action in the real world and confirms in the CLI, the CLI triggers a new FaaS invocation to resume the exact session with the human's feedback.

## Value Proposition
- **Physical World Access:** The Swarm can now orchestrate end-to-end hardware testing by treating the human developer as a "Sensor / Actuator".
- **Zero Idle Cost:** The Tachyon cluster consumes exactly 0 CPU cycles while waiting 5 minutes for the developer to put on their VR headset.