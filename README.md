# 🌌 Pulsar: Distributed Autonomous Engineering Swarm

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Host: Tachyon-Mesh](https://img.shields.io/badge/Host-Tachyon--Mesh-blueviolet)](https://github.com/astorise/tachyon-mesh)
[![Built with Rust](https://img.shields.io/badge/Language-Rust-orange)](https://www.rust-lang.org/)
[![Wasm Component Model](https://img.shields.io/badge/Runtime-WebAssembly-624de3)](https://component-model.bytecodealliance.org/)
[![Database RedDB](https://img.shields.io/badge/DB-RedDB-red)](https://github.com/crawfx/redb)

**Pulsar** est une plateforme d'ingénierie logicielle distribuée et autonome, conçue pour transformer des modèles de langage en une équipe d'ingénierie cohérente. S'appuyant sur l'infrastructure [Tachyon-Mesh](https://github.com/astorise/tachyon-mesh), Pulsar exécute des agents spécialisés sous forme de composants WebAssembly isolés, capables de collaborer massivement en parallèle tout en optimisant dynamiquement les ressources matérielles locales.

## 🚀 Vision : L'Empathie Synthétique

Pulsar n'est pas un scribe, c'est un **Collègue Dialectique**. Le système est bâti sur une méthodologie d'interaction rigoureuse visant à garantir l'alignement cognitif entre l'humain et la machine :
1.  **Context Handshake :** L'agent soumet un plan d'exécution structuré pour validation humaine avant toute modification du code.
2.  **Rabbit Hole Detection :** Détection automatique des boucles de raisonnement infructueuses, déclenchant la génération d'un rapport de situation (`SITUATION_REPORT.md`) plutôt que de persister dans l'erreur.
3.  **Human Action Bridge :** Suspension asynchrone à coût de calcul nul pour solliciter une action physique ou un jugement qualitatif de l'utilisateur.

---

## 🏗️ Architecture & Escalation Topology

Pulsar repose sur une architecture découplée, séparant l'Orchestrateur (CLI), le Middleware de Contexte (Viking), et un protocole strict d'escalade d'inférence à 3 niveaux.

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
| (Dell G15 3070ti)  |  | (Talos Cluster)    |  | (DeepSeek/Claude)  |
|--------------------|  |--------------------|  |--------------------|
| 7B Model + LoRA    |  | 27B Coder Model    |  | Hosted LLM         |
| Ultra-low latency  |  | Heavy reasoning    |  | Prompt Caching     |
| [ESCALATES DOUBT]  |  [ESCALATES ON FAIL]  |  |                    |
+--------------------+  +--------------------+  +--------------------+
        ^                         ^                         ^
        |                         |                         |
========|=========================|=========================|========
        |       Context Sharing & Semantic Compression      |
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
|    Monitors successful resolutions -> Generates SKILL.md        |
|    workflows to permanently improve Tier 1 & Tier 2 efficiency  |
+-----------------------------------------------------------------+
```

### 🧠 Le Workflow d'Exécution
1.  **Abstraction Viking :** Le système interroge le middleware pour obtenir une vue compressée du code (L1 AST).
2.  **Inférence Tiered :** Le Tier 1 (7B local) traite les tâches simples. En cas de doute, le Tier 2 (27B sur cluster Talos) prend le relais en utilisant le streaming de poids natif pour économiser la VRAM. Le Tier 3 (Cloud) est le filet de sécurité ultime.
3.  **Apprentissage Continu :** Chaque succès alimente le dataset local pour un Fine-Tuning LoRA natif, permettant au Tier 1 de gagner en expertise sur votre codebase spécifique.

---

## ⚙️ Capacités Avancées de l'Essaim

* **Supervisor (Map-Reduce) :** Décomposition des Epics en sous-tâches exécutées en parallèle dans des Git Worktrees isolés.
* **Paginated MCP Memory :** Mémoire partagée via RedDB paginée pour éviter les crashs OOM dans WebAssembly.
* **Cognitive Garbage Collector :** Résolution asynchrone des contradictions factuelles via des transactions atomiques (`atomic-swap`).
* **SmolVM Browser Agent :** Tests d'interfaces (CDP) isolés dans des MicroVMs jetables pour une sécurité totale.

---

## 🛠️ Inspirations et Remerciements

Pulsar est une synthèse architecturale des projets les plus novateurs de la communauté :

* **[OpenViking](https://github.com/volcengine/OpenViking) :** La fondation du Middleware de Contexte pour l'extraction sémantique et la gestion des niveaux L0-L2.
* **[gstack](https://github.com/garrytan/gstack) :** Pour le standard `SKILL.md` (Markdown-as-Code), le moteur de contradictions et l'automatisation CDP.
* **[Nous Hermes](https://github.com/NousResearch/Hermes-LLM) :** Pour les bases de modèles de raisonnement ayant inspiré nos protocoles d'escalade Tier 1 & 2.
* **[Kiln-AI](https://github.com/Kiln-AI/Kiln) :** Pour la structure du pipeline de génération de datasets à partir des traces d'exécution.
* **[Tachyon-Mesh](https://github.com/astorise/tachyon-mesh) :** Le socle d'infrastructure natif (FaaS, MicroVM, Training).

## ⚖️ Licence

Ce projet est distribué sous la **Licence MIT**.

---
*Développé avec passion par [astorise](https://github.com/astorise).*