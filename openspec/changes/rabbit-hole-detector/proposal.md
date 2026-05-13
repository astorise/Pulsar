# Proposal: Rabbit Hole Detector

## Problem Statement
AI agents often enter a "Rabbit Hole" (Anti-pattern #3) where they repeatedly try to solve a problem with slight, ineffective variations of the same tool calls. This leads to high token consumption, VRAM saturation, and developer frustration when the agent finally "gives up" without leaving a clear trail of why it failed.

## Vision
Inspired by the "Engineering Synthetic Empathy" framework, we will implement a loop detection logic in the `faas/orchestrator`. If the agent fails a specific goal (e.g., a test suite remains red) after $N$ attempts (default: 3), the agent MUST halt its automated execution. Instead of continuing, it will generate a "Situation Report" (format pivot) to explain its current mental map and hand over control to the Supervisor or the human developer.

## Value Proposition
- **Operational Reliability:** Prevents infinite loops and runaway costs.
- **Improved Handoff:** The developer doesn't have to read 50 lines of execution logs; they receive a structured summary of the failure.
- **Dialectical Posture:** Forces the agent to move from "Scribe" (doing) to "Architect" (explaining) when a threshold is hit.