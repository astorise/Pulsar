# Implementation Tasks

- [x] **Task 1: Define the WIT Contract**
  - Create `wit/browser/cdp.wit` with the resource `session` and its methods (`Maps`, `evaluate-js`, `take-screenshot`).
  
- [x] **Task 2: Implement Host CDP Client**
  - In `core-host/src/browser_bridge.rs`, implement a basic async WebSocket client capable of sending CDP JSON-RPC commands and parsing responses.

- [x] **Task 3: Implement SmolVM Connector**
  - Connect the `browser_bridge.rs` to the existing `system-faas-microvm-runner`.
  - Provide a script to build the minimal `rootfs` (Alpine + Chromium + Xvfb) required by `smolvm`.

- [x] **Task 4: Scaffold `faas/browser-agent`**
  - Create the new FaaS worker. 
  - Give it access to `tachyon:browser/cdp` and `tachyon:ai/inference`.
  
- [x] **Task 5: Update the Swarm Supervisor**
  - In Phase 11 (`faas/supervisor`), update the `Reduce` logic. If a sub-task involved UI changes (HTML/CSS/JS), the Supervisor MUST spawn a `faas/browser-agent` to take a screenshot and pass the Vision-LLM check before merging the code.