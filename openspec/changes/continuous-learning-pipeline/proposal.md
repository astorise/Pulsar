# Proposal: Continuous Learning Pipeline (Native Edge LoRA)

## Problem Statement
The Tier 1 model (7B) handles basic tasks but frequently escalates to Tier 2 (27B+) for complex architectural refactoring. While the `skill-extractor` generates Markdown files for the agent to read (RAG), the base weights of the 7B model remain ignorant of the project's specific conventions. 

## Vision
[cite_start]Instead of relying on external tools, we will leverage Tachyon-Mesh's native `Async Edge LoRA Training` capabilities. The `faas/skill-extractor` will format successful execution traces into a ShareGPT JSONL dataset. [cite_start]Once a threshold is reached, it will directly submit a fine-tuning job via the `tachyon:ai/training` WIT interface. [cite_start]The Tachyon host will handle the training asynchronously, utilizing VRAM spillover to system RAM to avoid OOM crashes.

## Value Proposition
- **Native & Air-Gapped:** Zero external dependencies. [cite_start]The entire pipeline (inference, dataset generation, and fine-tuning) runs strictly within the secure Tachyon-Mesh environment.
- **Zero-Cost Expertise:** The local Tier 1 model will gradually learn the codebase's specific macros and architectural patterns directly from the Swarm's successes.
- [cite_start]**Resource Safe:** The low-priority background training ensures real-time FaaS workers are not blocked during backpropagation.