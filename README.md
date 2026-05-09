## 🏗️ Architecture & Escalation Topology

Pulsar is built on a decoupled architecture, separating the Orchestrator (CLI), the Context Middleware, and a strict 3-Tier Inference Escalation protocol.

```text
+-----------------------------------------------------------------+
|                     Pulsar CLI (Orchestrator)                   |
|  [Parallel Worktrees]  [MCP Local Tools]  [Context Management]  |
+-----------------------------------------------------------------+
        |                         |                         |
   1. Request                2. Escalate               3. Escalate
        |                         |                         |
        v                         v                         v
+--------------------+  +--------------------+  +--------------------+
| TIER 1: Local Edge |  | TIER 2: Local Node |  | TIER 3: Cloud API  |
| (Dell G15 3070ti)  |  | (Talos 2x 3060)    |  | (DeepSeek/Claude)  |
|--------------------|  |--------------------|  |--------------------|
| 7B Model + LoRA    |  | 27B Coder Model    |  | Hosted LLM         |
| Ultra-low latency  |  | Heavy reasoning    |  | Prompt Caching     |
| [ESCALATES DOUBT]  |  [ESCALATES ON FAIL]  |  |                    |
+--------------------+  +--------------------+  +--------------------+
        ^                         ^                         ^
        |                         |                         |
========|=========================|=========================|========
        |      Context Sharing & Semantic Compression       |
        v                         v                         v
+-----------------------------------------------------------------+
|             Viking Context Middleware FaaS (RAG)                |
|      [ L0: Summaries ]  [ L1: AST/Signatures ]  [ L2: Raw ]     |
+-----------------------------------------------------------------+
                                  |
                           (Async Update)
                                  v
+-----------------------------------------------------------------+
|            Skill Extractor FaaS (Autonomous Learning)           |
|     Monitors successful resolutions -> Generates SKILL.md       |
|     workflows to permanently improve Tier 1 & Tier 2 efficiency |
+-----------------------------------------------------------------+
```

### 🧠 The Workflow

1. **Context Abstraction:** Whenever Pulsar CLI encounters a codebase, it queries the **Viking Context FaaS** to retrieve a highly compressed semantic view (L1 AST).
2. **Tier 1 (Edge):** The CLI sends this lightweight context to the local 7B model. If the local model identifies a complex pattern requiring more reasoning, its LoRA triggers an `[ESCALATE]` token.
3. **Tier 2 (Server):** The CLI routes the problem to the Tachyon-hosted 27B model on the Talos server. This model leverages the same Viking Context bus to request deeper raw code (L2) only where needed.
4. **Tier 3 (Cloud):** Ultimate fallback for impossible local tasks, minimizing cost via strict context filtering and prompt caching.
5. **Night-Shift Learning:** Once a problem is resolved, the **Skill Extractor FaaS** asynchronously analyzes the trace, generating reusable metadata (`SKILL.md`) to ensure the agent solves similar problems instantly next time.