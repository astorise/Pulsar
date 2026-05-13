# Implementation Tasks: Viking Searchable Index

- [x] **Task 1: Update Viking WIT Contract**
  - Modifier `faas/viking-context/wit/world.wit` pour ajouter la fonction `search` à l'interface `viking-context`.
  - **Note :** Si l'interface `kv-partition` ne permet pas encore le listage, ajouter une fonction `list-keys: func() -> result<list<string>, string>` pour permettre à Viking d'itérer sur les résumés L0.

- [x] **Task 2: Implement Search Logic in Viking**
  - Dans `faas/viking-context/src/lib.rs`, implémenter la fonction `search(query)`.
  - Parcourir les clés de la `kv-partition` pour récupérer les payloads L0 (résumés).
  - Effectuer un filtrage par mots-clés (insensible à la casse) sur le contenu des résumés.
  - Retourner la liste des URIs `viking://` correspondantes.

- [x] **Task 3: Proactive Crawler with Smart Filtering (Orchestrator)**
  - Dans `faas/orchestrator/src/lib.rs`, enrichir `start_session` avec un crawler asynchrone.
  - **Filtrage critique :** Intégrer la crate `ignore` (ou une logique équivalente via WebDAV) pour parser le `.gitignore`.
  - Exclure explicitement du scan : `node_modules/`, `target/`, `.git/`, et tout fichier/dossier caché.
  - Effectuer un `PROPFIND` récursif sur le workspace WebDAV.
  - Pour chaque fichier valide (`.rs`, `.toml`, `.md`, etc.), appeler silencieusement `viking_context::resolve(uri, l0-summary)` pour pré-remplir le cache Tachyon.

- [x] **Task 4: Add Search Tool to the Orchestrator**
  - Dans `faas/orchestrator/src/lib.rs`, ajouter la variante `SearchVikingContext { query: String }` à l'enum `ToolCall`.
  - Implémenter le bras de match correspondant : appeler `viking_context::search(query)` et enregistrer les URIs trouvées dans la trace d'observation de la session.

- [x] **Task 5: Refine the Meta-Prompt (Connect -> Ask -> Act)**
  - Mettre à jour la fonction `build_inference_prompt` pour inclure la capacité de recherche.
  - Ajouter une instruction système stricte : *"Before modifying code, ALWAYS use `search_viking_context` to find relevant files, then `read_viking_context` to understand their structure. Do not waste tokens listing directories manually."*