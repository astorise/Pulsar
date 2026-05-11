# Proposal: Pulsar VS Code Web Extension

## Problem Statement
While the new thin `pulsar-cli` provides an ultra-fast terminal experience, developers spend most of their time in the IDE. Competitors like GitHub Copilot integrate directly into the editor's sidebar. We need a native UI for Pulsar.

## Vision
Because the Pulsar "Brain" now lives entirely in the `faas/orchestrator` on the Tachyon Mesh, our VS Code extension does not need to bundle heavy binaries, Python scripts, or local models. 
We will build a **Pure Web Extension**. It will use standard browser WebSockets to connect directly to the Tachyon orchestrator. This guarantees compatibility not just with desktop VS Code, but with `vscode.dev` and remote environments.

## Value Proposition
- **Zero Install Friction:** Installs instantly, even in browser-based IDEs.
- **Context-Aware:** The extension can read the user's current highlighted code and active file, injecting it directly into the `user_message` WebSocket payload.
- **Unified Experience:** It uses the exact same WebSocket JSON protocol as the CLI, meaning the backend doesn't even know (or care) if the user is typing in a terminal or in VS Code.