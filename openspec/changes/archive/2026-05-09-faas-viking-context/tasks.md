# Tasks: faas/viking-context

## WIT Interface

- [x] Définir `interface storage-broker` avec `read-file` et `stat-file` (retourne `file-stat` avec `modified-secs`)
- [x] Définir `interface kv-partition` avec `get`, `set`, `delete`
- [x] Définir `interface viking-context` avec enum `context-level`, record `context-response` et fonction `resolve`
- [x] Composer le `world viking-context-world` avec les deux imports et l'export

## Implémentation Rust

- [x] Mettre en place la structure `#[cfg(target_arch = "wasm32")] mod component` pour isoler le glue WASM du code testable natif
- [x] Implémenter `strip_viking_prefix` (validation du schéma `viking://`)
- [x] Implémenter `build_cache_key` (format `v1:{path}:{level}:{mtime_secs}`)
- [x] Implémenter `estimate_tokens` (approximation chars/4)
- [x] Implémenter `build_summary` — L0 : liste des items publics via `syn` + compte de lignes
- [x] Implémenter `extract_rust_skeleton` — L1 : squelette AST via `syn` (signatures, structs, enums, traits, impls sans corps)
- [x] Implémenter la logique de cache dans `resolve` : stat → lookup kv-partition → read+compute → write best-effort
- [x] Court-circuiter le cache pour L2 (contenu brut, déjà géré par le page cache OS)
- [x] Appeler `export!(VikingContextImpl)` pour l'enregistrement WASM component

## Configuration Cargo

- [x] `crate-type = ["cdylib", "rlib"]` pour supporter `cargo test` natif et `cargo component build`
- [x] Dépendance `syn` inconditionnelle (logique testable native)
- [x] Dépendance `wit-bindgen` conditionnelle `[target.'cfg(target_arch = "wasm32")'.dependencies]`
- [x] Section `[package.metadata.component]` pour cargo-component

## Tests

- [x] Tests unitaires `strip_viking_prefix` (cas valide et invalide)
- [x] Tests unitaires `build_cache_key` (unicité par mtime, level, path ; préfixe `v1:`)
- [x] Tests unitaires `estimate_tokens`
- [x] Tests unitaires `build_summary` L0 (chemin, lignes, items publics uniquement)
- [x] Tests unitaires `extract_rust_skeleton` L1 (corps supprimés, signatures conservées, fallback sur input non parseable)

## CI GitHub Actions

- [x] Workflow `test.yml` : `cargo test --lib` + Clippy sur cible native (ubuntu-latest)
- [x] Workflow `build.yml` : `cargo component build --release` WASM + build CLI matrix Linux/Windows/macOS
