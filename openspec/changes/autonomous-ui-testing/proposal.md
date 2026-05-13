# Proposal: Autonomous UI Testing (Dual CDP Engine)

## Problem Statement
While our Swarm is highly proficient at backend logic, it is entirely "blind" to frontend output. When asked to build or refactor a UI component, the agent cannot verify if the CSS rendered correctly, if the layout is broken, or if a button is visibly misaligned. The human developer remains the sole visual validator.

## Vision
Drawing inspiration from `gstack`, we will equip Pulsar with a `faas/browser-agent`. This agent will act as the "Eyes" of the Swarm. It uses the Chrome DevTools Protocol (CDP) to drive a headless browser, navigate to the locally served dev-server, take screenshots, and use a local Vision-LLM (e.g., Qwen-VL) to evaluate the UI against the design spec.

Crucially, to support both rapid local development and secure cluster deployment, the Tachyon Host will offer a **Dual Execution Backend** for the browser:
1. **Local Mode (Fast):** Connects to an existing Chrome instance on the developer's host OS.
2. **SmolVM Mode (Secure/Cluster):** Leverages Tachyon's `system-faas-microvm-runner` to dynamically boot a lightweight MicroVM containing Alpine Linux + Chromium. The agent controls this isolated browser, takes the screenshot, and the VM is instantly destroyed.

## Value Proposition
- **Full-Stack Autonomy:** The swarm can iterate on UI bugs without human intervention.
- **Zero-Pollution (SmolVM):** The agent cannot accidentally modify the developer's personal browser profiles, cookies, or history.
- **Air-Gapped Security:** The MicroVM has strictly controlled egress network policies, preventing malicious code in a pulled repo from exfiltrating data during the UI test.