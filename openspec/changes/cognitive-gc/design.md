# Design: Contradiction Engine Architecture

## 1. The Background Sweeper
The `cognitive-gc` FaaS is instantiated by a cron-trigger or runs as a low-priority background loop on the Tachyon host. 
It opens the `mcp_observations` table using the V2 `resource table` interface.

## 2. Semantic Clustering (Finding Conflicts)
To find contradictions, the engine must know which facts are talking about the same subject.
- The engine uses the `tachyon:ai/inference` WIT interface to generate lightweight embeddings (e.g., using a small local model like `all-MiniLM-L6-v2`) for every new observation.
- It calculates Cosine Similarity between recent facts. Highly similar facts are grouped into a "Collision Cluster".

## 3. The LLM Judge (Evaluation)
For each Collision Cluster, the engine builds a prompt for the Tier 2 model:

> **System Prompt:**
> You are the Memory Judge. Analyze these clustered facts:
> Fact A (Monday): "The database is SQLite."
> Fact B (Tuesday): "The database was migrated to RedDB."
>
> Are these facts in contradiction? If yes, analyze the timestamps and the attached Git commits to determine the prevailing truth.
> Output a strict JSON: { "has_contradiction": true, "golden_fact": "The database uses RedDB (migrated from SQLite on Tuesday)." }

## 4. The Atomic Resolution (RedDB)
If a contradiction is confirmed and a Golden Fact is synthesized, the FaaS executes an atomic update via the `kv-partition` WIT:
1. `delete(Fact A)`
2. `delete(Fact B)`
3. `set(New Golden Fact)`