// ── WASM component glue (wasm32 only) ────────────────────────────────────────
// Not compiled on native so `cargo test` can exercise the pure-logic layer
// without needing the Tachyon runtime.
#[cfg(target_arch = "wasm32")]
mod component {
    wit_bindgen::generate!({
        path: "wit",
        world: "viking-context-world",
    });

    use exports::tachyon::ai::viking_context::{ContextLevel, ContextResponse, Guest};
    use tachyon::ai::{graph, kv_partition, storage_broker};

    struct VikingContextImpl;

    impl Guest for VikingContextImpl {
        fn resolve(uri: String, level: ContextLevel) -> Result<ContextResponse, String> {
            let path = super::strip_viking_prefix(&uri)?;

            // L2 is the raw file — no AST work, no benefit to caching.
            if matches!(level, ContextLevel::L2Raw) {
                let bytes = storage_broker::read_file(&path)?;
                let payload =
                    String::from_utf8(bytes).map_err(|_| format!("'{path}' is not valid UTF-8"))?;
                return Ok(ContextResponse {
                    uri,
                    level,
                    token_estimate: super::estimate_tokens(&payload),
                    payload,
                });
            }

            // Stat the file to build an mtime-keyed cache entry.
            // If stat fails (e.g. path does not exist), propagate the error now
            // rather than doing the expensive read only to fail later.
            let stat = storage_broker::stat_file(&path)?;
            let level_tag = match level {
                ContextLevel::L0Summary => "l0",
                ContextLevel::L1Structure => "l1",
                ContextLevel::L2Raw => unreachable!(),
            };
            let cache_key = super::build_cache_key(&path, stat.modified_secs, level_tag);

            // ── Cache hit ────────────────────────────────────────────────────
            if let Ok(Some(cached)) = kv_partition::get(&cache_key) {
                if let Ok(payload) = String::from_utf8(cached) {
                    return Ok(ContextResponse {
                        uri,
                        level,
                        token_estimate: super::estimate_tokens(&payload),
                        payload,
                    });
                }
                // Corrupted entry: fall through and recompute.
            }

            // ── Cache miss: read file and compute payload ─────────────────────
            let bytes = storage_broker::read_file(&path)?;
            let raw =
                String::from_utf8(bytes).map_err(|_| format!("'{path}' is not valid UTF-8"))?;

            let (payload, resolved) = match level {
                ContextLevel::L0Summary => {
                    (super::build_summary(&path, &raw), ContextLevel::L0Summary)
                }
                ContextLevel::L1Structure => {
                    if path.ends_with(".rs") {
                        let edges = super::extract_rust_graph_edges(&path, &raw);
                        commit_graph_edges(&edges);
                        (
                            super::extract_rust_skeleton(&raw),
                            ContextLevel::L1Structure,
                        )
                    } else {
                        // Non-Rust files: return raw, note the fallback in level.
                        (raw, ContextLevel::L2Raw)
                    }
                }
                ContextLevel::L2Raw => unreachable!(),
            };

            // Persist to cache (best-effort: a write failure must not break the call).
            let _ = kv_partition::set(&cache_key, payload.as_bytes());

            Ok(ContextResponse {
                uri,
                level: resolved,
                token_estimate: super::estimate_tokens(&payload),
                payload,
            })
        }

        fn search(query: String) -> Result<Vec<String>, String> {
            let keys = kv_partition::list_keys()?;
            let mut matches = Vec::new();

            for key in keys {
                let Some(path) = super::cache_key_path_for_level(&key, "l0") else {
                    continue;
                };
                let Some(bytes) = kv_partition::get(&key)? else {
                    continue;
                };
                let Ok(summary) = String::from_utf8(bytes) else {
                    continue;
                };
                if super::summary_matches_query(&summary, &query) {
                    matches.push(format!("viking://{path}"));
                }
            }

            matches.sort();
            matches.dedup();
            Ok(matches)
        }

        fn graph_query(entity: String, depth: u32) -> Result<Vec<String>, String> {
            let graph = graph::WorkspaceGraph::new("viking_graph");
            let mut results = graph.traverse(&entity, "references", depth)?;
            if let Ok(mut imported_by) = graph.traverse(&entity, "imports", depth) {
                results.append(&mut imported_by);
            }
            if let Ok(mut implemented_by) = graph.traverse(&entity, "implements", depth) {
                results.append(&mut implemented_by);
            }
            results.sort();
            results.dedup();
            Ok(results)
        }
    }

    fn commit_graph_edges(edges: &[super::GraphEdge]) {
        if edges.is_empty() {
            return;
        }

        let graph = graph::WorkspaceGraph::new("viking_graph");
        let batch = edges
            .iter()
            .map(|edge| graph::Edge {
                subject: edge.subject.clone(),
                predicate: edge.predicate.clone(),
                object: edge.object.clone(),
                properties: edge.properties.clone(),
            })
            .collect::<Vec<_>>();
        let _ = graph.add_edges(&batch);
    }

    export!(VikingContextImpl);
}

// ── Pure business logic (always compiled, tested on native) ──────────────────

use std::collections::BTreeSet;

/// Strip the `viking://` scheme and return the bare file path.
pub fn strip_viking_prefix(uri: &str) -> Result<String, String> {
    uri.strip_prefix("viking://")
        .map(str::to_string)
        .ok_or_else(|| format!("invalid viking URI: expected 'viking://' prefix, got '{uri}'"))
}

/// Build a versioned, mtime-keyed cache entry identifier.
///
/// Key format: `v1:<path>:<level>:<mtime_secs>`
/// The `v1:` prefix allows future format migrations without explicit eviction:
/// old entries are simply never matched and expire naturally.
pub fn build_cache_key(path: &str, mtime_secs: u64, level: &str) -> String {
    format!("v1:{path}:{level}:{mtime_secs}")
}

pub fn cache_key_path_for_level(key: &str, expected_level: &str) -> Option<String> {
    let rest = key.strip_prefix("v1:")?;
    let (path_and_level, _mtime) = rest.rsplit_once(':')?;
    let (path, level) = path_and_level.rsplit_once(':')?;
    if level == expected_level {
        Some(path.to_string())
    } else {
        None
    }
}

pub fn summary_matches_query(summary: &str, query: &str) -> bool {
    let summary = summary.to_ascii_lowercase();
    let tokens = query
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();

    !tokens.is_empty() && tokens.iter().all(|token| summary.contains(token))
}

/// Rough token estimate: 4 ASCII chars ≈ 1 token.
pub fn estimate_tokens(text: &str) -> u32 {
    u32::try_from(text.len() / 4).unwrap_or(u32::MAX)
}

/// L0 — one-paragraph summary suitable for bulk scanning by the agent.
pub fn build_summary(path: &str, content: &str) -> String {
    let line_count = content.lines().count();
    let mut out = format!("# {path}\nLines: {line_count}\n");

    if path.ends_with(".rs") {
        let Ok(file) = syn::parse_file(content) else {
            return out;
        };
        let pub_items: Vec<String> = file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Fn(f) if matches!(f.vis, syn::Visibility::Public(_)) => {
                    Some(format!("fn {}", f.sig.ident))
                }
                syn::Item::Struct(s) if matches!(s.vis, syn::Visibility::Public(_)) => {
                    Some(format!("struct {}", s.ident))
                }
                syn::Item::Enum(e) if matches!(e.vis, syn::Visibility::Public(_)) => {
                    Some(format!("enum {}", e.ident))
                }
                syn::Item::Trait(t) if matches!(t.vis, syn::Visibility::Public(_)) => {
                    Some(format!("trait {}", t.ident))
                }
                _ => None,
            })
            .collect();

        if !pub_items.is_empty() {
            out.push_str("Public: ");
            out.push_str(&pub_items.join(", "));
            out.push('\n');
        }
    }

    out
}

/// L1 — AST skeleton for Rust files: declarations without bodies.
/// Falls back to raw source if `syn` cannot parse the input.
pub fn extract_rust_skeleton(src: &str) -> String {
    let Ok(file) = syn::parse_file(src) else {
        return src.to_string();
    };

    let mut out = String::new();
    for item in &file.items {
        write_item_skeleton(&mut out, item);
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphEdge {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub properties: String,
}

impl GraphEdge {
    pub fn new(subject: String, predicate: &str, object: String, properties: String) -> Self {
        Self {
            subject,
            predicate: predicate.to_string(),
            object,
            properties,
        }
    }
}

pub fn extract_rust_graph_edges(path: &str, src: &str) -> Vec<GraphEdge> {
    let Ok(file) = syn::parse_file(src) else {
        return Vec::new();
    };

    let module = module_urn_for_path(path);
    let mut visitor = GraphVisitor {
        file: format!("file:{path}"),
        module,
        edges: BTreeSet::new(),
    };
    syn::visit::Visit::visit_file(&mut visitor, &file);
    visitor.edges.into_iter().collect()
}

pub fn diff_graph_edges(old: &[GraphEdge], new: &[GraphEdge]) -> (Vec<GraphEdge>, Vec<GraphEdge>) {
    let old = old.iter().cloned().collect::<BTreeSet<_>>();
    let new = new.iter().cloned().collect::<BTreeSet<_>>();
    let deletes = old.difference(&new).cloned().collect();
    let inserts = new.difference(&old).cloned().collect();
    (deletes, inserts)
}

pub fn module_urn_for_path(path: &str) -> String {
    let normalized = path
        .trim_end_matches(".rs")
        .trim_end_matches("/mod")
        .trim_end_matches("\\mod")
        .replace('\\', "/");
    let module = normalized
        .trim_start_matches("src/")
        .trim_start_matches("./")
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("::");
    if module.is_empty() {
        "module:crate".to_string()
    } else {
        format!("module:{module}")
    }
}

struct GraphVisitor {
    file: String,
    module: String,
    edges: BTreeSet<GraphEdge>,
}

impl GraphVisitor {
    fn push(&mut self, subject: String, predicate: &str, object: String, visibility: &str) {
        self.edges.insert(GraphEdge::new(
            subject,
            predicate,
            object,
            format!(r#"{{"line":0,"visibility":"{visibility}"}}"#),
        ));
    }

    fn contains(&mut self, object: String, visibility: &syn::Visibility) {
        let module = self.module.clone();
        self.push(module, "contains", object, visibility_label(visibility));
    }
}

impl<'ast> syn::visit::Visit<'ast> for GraphVisitor {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        for path in use_tree_paths(&node.tree) {
            let module = self.module.clone();
            self.push(
                module,
                "imports",
                format!("struct:{}", path.join("::")),
                visibility_label(&node.vis),
            );
        }
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let struct_urn = format!(
            "struct:{}::{}",
            self.module.trim_start_matches("module:"),
            node.ident
        );
        self.contains(struct_urn.clone(), &node.vis);
        self.push(
            self.file.clone(),
            "contains",
            struct_urn.clone(),
            visibility_label(&node.vis),
        );
        for field in &node.fields {
            if let Some(name) = type_path_name(&field.ty) {
                self.push(
                    struct_urn.clone(),
                    "references",
                    format!("struct:{name}"),
                    visibility_label(&field.vis),
                );
            }
        }
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        let enum_urn = format!(
            "enum:{}::{}",
            self.module.trim_start_matches("module:"),
            node.ident
        );
        self.contains(enum_urn.clone(), &node.vis);
        self.push(
            self.file.clone(),
            "contains",
            enum_urn,
            visibility_label(&node.vis),
        );
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let trait_urn = format!(
            "trait:{}::{}",
            self.module.trim_start_matches("module:"),
            node.ident
        );
        self.contains(trait_urn.clone(), &node.vis);
        self.push(
            self.file.clone(),
            "contains",
            trait_urn,
            visibility_label(&node.vis),
        );
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if let Some((_, trait_path, _)) = &node.trait_ {
            let Some(self_ty) = type_path_name(&node.self_ty) else {
                syn::visit::visit_item_impl(self, node);
                return;
            };
            let Some(trait_name) = trait_path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .reduce(|mut acc, next| {
                    acc.push_str("::");
                    acc.push_str(&next);
                    acc
                })
            else {
                syn::visit::visit_item_impl(self, node);
                return;
            };
            self.push(
                format!(
                    "struct:{}::{}",
                    self.module.trim_start_matches("module:"),
                    self_ty
                ),
                "implements",
                format!("trait:{trait_name}"),
                "private",
            );
        }
        syn::visit::visit_item_impl(self, node);
    }
}

fn write_item_skeleton(out: &mut String, item: &syn::Item) {
    match item {
        syn::Item::Fn(f) => {
            let async_kw = if f.sig.asyncness.is_some() {
                "async "
            } else {
                ""
            };
            let inputs: Vec<String> = f
                .sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    syn::FnArg::Receiver(_) => "self".to_string(),
                    syn::FnArg::Typed(pt) => {
                        if let syn::Pat::Ident(pi) = pt.pat.as_ref() {
                            pi.ident.to_string()
                        } else {
                            "_".to_string()
                        }
                    }
                })
                .collect();
            out.push_str(&format!(
                "{async_kw}fn {}({}) {{ /* ... */ }}\n\n",
                f.sig.ident,
                inputs.join(", ")
            ));
        }

        syn::Item::Struct(s) => {
            out.push_str(&format!("struct {} {{\n", s.ident));
            for field in &s.fields {
                if let Some(ident) = &field.ident {
                    out.push_str(&format!("    {ident}: _,\n"));
                }
            }
            out.push_str("}\n\n");
        }

        syn::Item::Enum(e) => {
            out.push_str(&format!("enum {} {{\n", e.ident));
            for variant in &e.variants {
                out.push_str(&format!("    {},\n", variant.ident));
            }
            out.push_str("}\n\n");
        }

        syn::Item::Trait(t) => {
            out.push_str(&format!("trait {} {{\n", t.ident));
            for ti in &t.items {
                if let syn::TraitItem::Fn(m) = ti {
                    let async_kw = if m.sig.asyncness.is_some() {
                        "async "
                    } else {
                        ""
                    };
                    out.push_str(&format!("    {async_kw}fn {}(...);\n", m.sig.ident));
                }
            }
            out.push_str("}\n\n");
        }

        syn::Item::Impl(i) => {
            let self_ty = extract_type_name(&i.self_ty);
            let header = if let Some((_, path, _)) = &i.trait_ {
                let trait_name = path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                format!("impl {trait_name} for {self_ty}")
            } else {
                format!("impl {self_ty}")
            };
            out.push_str(&format!("{header} {{\n"));
            for ii in &i.items {
                if let syn::ImplItem::Fn(m) = ii {
                    let async_kw = if m.sig.asyncness.is_some() {
                        "async "
                    } else {
                        ""
                    };
                    out.push_str(&format!(
                        "    {async_kw}fn {}(...) {{ /* ... */ }}\n",
                        m.sig.ident
                    ));
                }
            }
            out.push_str("}\n\n");
        }

        syn::Item::Mod(m) => {
            out.push_str(&format!("mod {} {{ /* ... */ }}\n\n", m.ident));
        }

        syn::Item::Type(t) => {
            out.push_str(&format!("type {} = _;\n\n", t.ident));
        }

        syn::Item::Const(c) => {
            out.push_str(&format!("const {}: _ = _;\n\n", c.ident));
        }

        _ => {}
    }
}

fn extract_type_name(ty: &syn::Type) -> String {
    if let syn::Type::Path(p) = ty {
        p.path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| "?".to_string())
    } else {
        "?".to_string()
    }
}

fn type_path_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(p) => p
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        syn::Type::Reference(reference) => type_path_name(&reference.elem),
        _ => None,
    }
}

fn visibility_label(vis: &syn::Visibility) -> &'static str {
    match vis {
        syn::Visibility::Public(_) => "public",
        _ => "private",
    }
}

fn use_tree_paths(tree: &syn::UseTree) -> Vec<Vec<String>> {
    fn walk(prefix: &mut Vec<String>, tree: &syn::UseTree, out: &mut Vec<Vec<String>>) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                walk(prefix, &path.tree, out);
                prefix.pop();
            }
            syn::UseTree::Name(name) => {
                let mut path = prefix.clone();
                path.push(name.ident.to_string());
                out.push(path);
            }
            syn::UseTree::Rename(rename) => {
                let mut path = prefix.clone();
                path.push(rename.ident.to_string());
                out.push(path);
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    walk(prefix, item, out);
                }
            }
            syn::UseTree::Glob(_) => {
                let mut path = prefix.clone();
                path.push("*".to_string());
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    walk(&mut Vec::new(), tree, &mut out);
    out
}

// ── Unit tests (native target) ────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    // ── strip_viking_prefix ───────────────────────────────────────────────────

    #[test]
    fn viking_prefix_valid() {
        assert_eq!(
            strip_viking_prefix("viking://src/main.rs").unwrap(),
            "src/main.rs"
        );
    }

    #[test]
    fn viking_prefix_missing_scheme() {
        assert!(strip_viking_prefix("src/main.rs").is_err());
    }

    // ── build_cache_key ───────────────────────────────────────────────────────

    #[test]
    fn cache_key_contains_all_components() {
        let key = build_cache_key("src/main.rs", 1_700_000_000, "l1");
        assert!(key.contains("src/main.rs"));
        assert!(key.contains("1700000000"));
        assert!(key.contains("l1"));
    }

    #[test]
    fn cache_key_has_version_prefix() {
        let key = build_cache_key("src/main.rs", 0, "l0");
        assert!(key.starts_with("v1:"));
    }

    #[test]
    fn cache_key_differs_on_mtime() {
        let k1 = build_cache_key("src/main.rs", 100, "l1");
        let k2 = build_cache_key("src/main.rs", 101, "l1");
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_differs_on_level() {
        let k1 = build_cache_key("src/main.rs", 100, "l0");
        let k2 = build_cache_key("src/main.rs", 100, "l1");
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_differs_on_path() {
        let k1 = build_cache_key("src/main.rs", 100, "l1");
        let k2 = build_cache_key("src/lib.rs", 100, "l1");
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_path_extracts_only_requested_level() {
        assert_eq!(
            cache_key_path_for_level("v1:src/lib.rs:l0:170", "l0"),
            Some("src/lib.rs".to_string())
        );
        assert_eq!(cache_key_path_for_level("v1:src/lib.rs:l1:170", "l0"), None);
    }

    #[test]
    fn summary_search_matches_all_query_tokens_case_insensitively() {
        assert!(summary_matches_query(
            "Public: struct SessionConfig, fn run",
            "session config"
        ));
        assert!(!summary_matches_query(
            "Public: struct SessionConfig",
            "session missing"
        ));
    }

    // ── estimate_tokens ───────────────────────────────────────────────────────

    #[test]
    fn token_estimate_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn token_estimate_approx() {
        assert_eq!(estimate_tokens("abcdefgh"), 2);
    }

    // ── build_summary (L0) ────────────────────────────────────────────────────

    #[test]
    fn l0_contains_path_and_lines() {
        let summary = build_summary("src/main.rs", "fn main() {}\n");
        assert!(summary.contains("src/main.rs"));
        assert!(summary.contains("Lines: 1"));
    }

    #[test]
    fn l0_lists_public_items_only() {
        let src = r#"
pub struct Config { pub name: String }
pub fn run() {}
fn private() {}
pub enum Mode { Fast, Slow }
"#;
        let summary = build_summary("src/lib.rs", src);
        assert!(summary.contains("struct Config"));
        assert!(summary.contains("fn run"));
        assert!(summary.contains("enum Mode"));
        assert!(!summary.contains("private"));
    }

    // ── extract_rust_skeleton (L1) ────────────────────────────────────────────

    #[test]
    fn l1_strips_function_bodies() {
        let src = r#"
pub fn process(config: &Config) -> Result<(), String> {
    println!("{}", config.name);
    Ok(())
}
"#;
        let skeleton = extract_rust_skeleton(src);
        assert!(skeleton.contains("fn process"));
        assert!(!skeleton.contains("println!"));
        assert!(!skeleton.contains("Ok(())"));
    }

    #[test]
    fn l1_keeps_struct_fields() {
        let src = "pub struct Config { pub name: String, value: u32, }";
        let skeleton = extract_rust_skeleton(src);
        assert!(skeleton.contains("struct Config"));
        assert!(skeleton.contains("name"));
        assert!(skeleton.contains("value"));
    }

    #[test]
    fn l1_keeps_enum_variants() {
        let src = "pub enum Status { Active, Inactive, Pending }";
        let skeleton = extract_rust_skeleton(src);
        assert!(skeleton.contains("enum Status"));
        assert!(skeleton.contains("Active"));
        assert!(skeleton.contains("Inactive"));
    }

    #[test]
    fn l1_keeps_trait_signatures() {
        let src = r#"
pub trait Handler {
    fn handle(&self, input: &str) -> String;
    async fn handle_async(&self) -> String;
}
"#;
        let skeleton = extract_rust_skeleton(src);
        assert!(skeleton.contains("trait Handler"));
        assert!(skeleton.contains("fn handle"));
        assert!(skeleton.contains("async fn handle_async"));
    }

    #[test]
    fn l1_strips_impl_bodies() {
        let src = r#"
impl Handler for Config {
    fn handle(&self, input: &str) -> String {
        self.name.clone() + input
    }
}
"#;
        let skeleton = extract_rust_skeleton(src);
        assert!(skeleton.contains("impl Handler for Config"));
        assert!(skeleton.contains("fn handle"));
        assert!(!skeleton.contains("name.clone()"));
    }

    #[test]
    fn l1_fallback_on_unparseable_input() {
        let raw = "not valid rust {{ {{ {";
        assert_eq!(extract_rust_skeleton(raw), raw);
    }

    #[test]
    fn graph_edges_capture_imports_definitions_and_impls() {
        let src = r#"
use crate::domain::Repository;
pub struct UserRepo { repo: Repository }
pub trait Handler {}
impl Handler for UserRepo {}
"#;
        let edges = extract_rust_graph_edges("src/domain/user_repo.rs", src);

        assert!(edges.iter().any(|edge| edge.predicate == "imports"));
        assert!(edges.iter().any(|edge| edge.object.ends_with("::UserRepo")));
        assert!(edges.iter().any(|edge| edge.predicate == "implements"));
    }

    #[test]
    fn graph_edge_diff_returns_deletes_and_inserts() {
        let old = vec![GraphEdge::new(
            "a".to_string(),
            "contains",
            "b".to_string(),
            "{}".to_string(),
        )];
        let new = vec![GraphEdge::new(
            "a".to_string(),
            "contains",
            "c".to_string(),
            "{}".to_string(),
        )];
        let (deletes, inserts) = diff_graph_edges(&old, &new);

        assert_eq!(deletes[0].object, "b");
        assert_eq!(inserts[0].object, "c");
    }
}
