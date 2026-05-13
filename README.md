# 🌌 Pulsar: Distributed Autonomous Engineering Swarm

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Host: Tachyon-Mesh](https://img.shields.io/badge/Host-Tachyon--Mesh-blueviolet)](https://github.com/astorise/tachyon-mesh)
[![Built with Rust](https://img.shields.io/badge/Language-Rust-orange)](https://www.rust-lang.org/)
[![Wasm Component Model](https://img.shields.io/badge/Runtime-WebAssembly-624de3)](https://component-model.bytecodealliance.org/)
[![Database RedDB](https://img.shields.io/badge/DB-RedDB-red)](https://github.com/crawfx/redb)

**Pulsar** est une plateforme d'ingénierie logicielle distribuée et autonome, conçue pour transformer des modèles de langage en une équipe d'ingénierie cohérente. S'appuyant sur l'infrastructure [Tachyon-Mesh](https://github.com/astorise/tachyon-mesh), Pulsar exécute des agents spécialisés sous forme de composants WebAssembly isolés, capables de collaborer massivement en parallèle tout en optimisant dynamiquement les ressources matérielles locales.

## 🚀 Vision : L'Empathie Synthétique

Pulsar ne considère pas l'IA comme un simple "Scribe", mais comme un **Collègue Dialectique**. Le système est bâti sur une méthodologie visant à garantir la fiabilité opérationnelle et l'alignement cognitif entre l'humain et la machine :
1.  **Context Handshake :** L'agent doit soumettre un plan d'exécution structuré pour validation humaine avant toute modification du code.
2.  **Rabbit Hole Detection :** Détection automatique des boucles de raisonnement infructueuses, déclenchant la génération d'un rapport de situation (`SITUATION_REPORT.md`) et un "Handoff" (passage de relais) plutôt que de persister dans l'erreur.
3.  **Human Action Bridge :** Suspension asynchrone de l'agent à coût de calcul nul pour solliciter une action physique (ex: brancher un device) de la part de l'humain.

---

## 🏗️ Architecture & Escalation Topology

Pulsar repose sur une architecture découplée, séparant l'Orchestrateur (CLI), le Middleware de Contexte sémantique (Viking), et un protocole strict d'escalade d'inférence à 3 niveaux pour préserver les ressources et le budget.

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

1. **Abstraction de Contexte :** Lorsque le CLI Pulsar interagit avec la base de code, il interroge le **Viking Context FaaS** pour récupérer une vue sémantique hautement compressée (L1 AST/Graphe de dépendances).
2. **Tier 1 (Edge) :** Le CLI envoie ce contexte allégé au modèle local 7B. Si le modèle identifie un motif complexe dépassant ses capacités, son réseau LoRA déclenche un token `[ESCALATE]`.
3. **Tier 2 (Server) :** Le CLI route le problème vers le modèle 27B+ hébergé sur le cluster local. Ce modèle utilise l'inférence native Tachyon en **Layer-Wise Streaming** (`mmap`) pour faire tourner des poids massifs en contournant les limites de VRAM. Il interroge Viking pour obtenir du code brut profond (L2) uniquement si nécessaire.
4. **Tier 3 (Cloud) :** Le filet de sécurité ultime pour les tâches d'architecture impossibles en local, minimisant les coûts grâce au *Prompt Caching*.
5. **Night-Shift Learning :** Une fois le problème résolu, la FaaS **Skill Extractor** analyse la trace de manière asynchrone pour générer des métadonnées (`SKILL.md`) et forger des datasets d'entraînement, garantissant que l'agent local (Tier 1) résoudra ce problème sans escalade la prochaine fois.

---

## ⚙️ Composants Avancés de l'Essaim (Swarm)

Au-delà de la topologie matérielle, Pulsar déploie des fonctionnalités logicielles avancées pour gérer la collaboration multi-agents :

* **Supervisor (Map-Reduce) :** Capacité à décomposer une "Epic" en sous-tâches exécutées en parallèle par des `faas/orchestrator` éphémères dans des Git Worktrees isolés.
* **Paginated MCP Memory :** Utilisation du Wasm Component Model (`resource table` sur RedDB) pour offrir à l'essaim une mémoire partagée (RAG) ultra-rapide sans provoquer de crashs OOM (Out-Of-Memory).
* **Cognitive Garbage Collector :** Un processus d'arrière-plan qui scanne vectoriellement les observations de l'essaim et utilise un LLM "Juge" pour résoudre les contradictions (ex: *Fact A vs Fact B*) via des transactions RedDB atomiques (`atomic-swap`).
* **SmolVM Browser Agent :** Pour le test d'interfaces graphiques, Pulsar instancie une MicroVM (Alpine + Chromium headless) éphémère pilotée par CDP. L'agent analyse le rendu visuel (Vision-LLM) sans aucun risque de sécurité ou de pollution d'état pour l'hôte.

---

## 🛠️ Inspirations et Remerciements

L'architecture de Pulsar synthétise les concepts les plus innovants de l'écosystème Open Source actuel :

* **[gstack](https://github.com/garrytan/gstack) :** Pour l'approche `SKILL.md` (Markdown-as-Code avec routage vectoriel sémantique), l'intégration du CDP et l'idée originelle du moteur de résolution des contradictions temporelles.
* **[Kiln-AI](https://github.com/Kiln-AI/Kiln) :** Pour l'automatisation de la boucle d'auto-amélioration continue (filtration des traces d'exécution en datasets Alpaca/ShareGPT purifiés).
* **[AirLLM](https://github.com/lyogavin/AirLLM) :** Pour la preuve mathématique de l'inférence *Layer-Wise* (chargement/déchargement couche par couche), réécrite ici nativement en Rust via [Candle](https://github.com/huggingface/candle) pour un contournement matériel des limites VRAM.
* **[Tachyon-Mesh](https://github.com/astorise/tachyon-mesh) :** Le socle d'infrastructure natif fournissant le moteur d'exécution Wasm, le bus réseau et les connecteurs RedDB.

---

## ⚖️ Licence

Ce projet est distribué sous la **Licence MIT**. Voir le fichier [LICENSE](LICENSE) pour plus de détails.

---
*Développé avec passion par [astorise](https://github.com/astorise).*