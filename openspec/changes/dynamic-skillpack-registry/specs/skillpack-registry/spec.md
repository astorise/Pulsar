# Specification: Skillpack Registry Protocol

## Requirement: Markdown-as-Code
The system SHALL support the definition of agent personas, instructions, and tool access control lists (ACLs) exclusively through Markdown files adorned with YAML frontmatter.

## Requirement: Pre-Compiled Cache
The Host environment MUST NOT read `SKILL.md` files from the disk during the critical path of a FaaS invocation. All skills MUST be retrieved from the pre-compiled `resource table` in RedDB.

## Requirement: Vector-Based Routing
The Supervisor FaaS SHALL implement an $O(N)$ or HNSW vector search against the cached skill embeddings to select the appropriate sub-agent for a delegated task, acting as a dynamic Semantic Router.