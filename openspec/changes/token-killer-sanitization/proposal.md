# Proposal: Embedded Token Killer

## Problem Statement
Command-line tools generate outputs meant for humans (ANSI colors, progress bars, repetitive warnings). Sending this raw data to the Tier 2 LLM wastes VRAM and context window limits.

## Vision
We will embed a lightweight, data-driven sanitization module directly into the `faas/orchestrator`. This module uses a JSON configuration file containing Regex filters inspired by the `rtk-ai/rtk` project. It will purify CLI outputs before they are added to the agent's execution trace.

## Value Proposition
- **Zero-Cost Abstraction:** By using `serde_json` and `LazyLock`, regex compilation happens once at startup. Runtime penalty is ~0.
- **WASM Compatible:** Pure Rust regex filters maintain strict compatibility with `wasm32-wasip2`.
- **High Compression:** Reduces raw command outputs by 70% to 90%.