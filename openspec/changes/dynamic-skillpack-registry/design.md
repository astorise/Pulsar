# Design: Skillpack Pre-Processing and Routing

## 1. The `SKILL.md` Standard
A skill file must contain a YAML Frontmatter for metadata/tools, and Markdown for the system prompt.
```markdown
---
name: "db-migrator"
description: "Generates and reviews SQL migration scripts for RedDB."
tools: ["read_schema", "write_sql"]
---
You are the Database Migration Specialist...
```

## 2. The `pulsar-cli skillify` Compiler
A new command in the CLI acts as the pre-processor.
1. It reads all `SKILL.md` files.
2. It calls the `tachyon:ai/inference.embed` WIT interface to generate a vector `[f32; 384]` of the `description`.
3. It serializes the parsed YAML and Markdown into a compressed binary format (e.g., MessagePack).
4. It executes a `batch-set` into the `pulsar_skill_registry` RedDB table.

## 3. RAG-Powered Supervisor Routing
When the Supervisor (Phase 11) analyzes a user prompt (e.g., "Add a user_id column to the sessions table"):
1. The Supervisor generates an embedding of the user's task.
2. It queries the `pulsar_skill_registry` using Cosine Similarity.
3. It discovers the `db-migrator` skill as the top match.
4. It dynamically spawns an orchestrator FaaS, injecting the pre-compiled `db-migrator` binary payload directly into its context.