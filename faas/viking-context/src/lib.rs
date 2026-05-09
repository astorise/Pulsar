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
    use tachyon::ai::{kv_partition, storage_broker};

    struct VikingContextImpl;

    impl Guest for VikingContextImpl {
        fn resolve(uri: String, level: ContextLevel) -> Result<ContextResponse, String> {
            let path = super::strip_viking_prefix(&uri)?;

            // L2 is the raw file — no AST work, no benefit to caching.
            if matches!(level, ContextLevel::L2Raw) {
                let bytes = storage_broker::read_file(&path)?;
                let payload = String::from_utf8(bytes)
                    .map_err(|_| format!("'{path}' is not valid UTF-8"))?;
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
            let raw = String::from_utf8(bytes)
                .map_err(|_| format!("'{path}' is not valid UTF-8"))?;

            let (payload, resolved) = match level {
                ContextLevel::L0Summary => {
                    (super::build_summary(&path, &raw), ContextLevel::L0Summary)
                }
                ContextLevel::L1Structure => {
                    if path.ends_with(".rs") {
                        (super::extract_rust_skeleton(&raw), ContextLevel::L1Structure)
                    } else {
                        // Non-Rust files: return raw, note the fallback in level.
                        (raw, ContextLevel::L2Raw)
                    }
                }
                ContextLevel::L2Raw => unreachable!(),
            };

            // Persist to cache (best-effort: a write failure must not break the call).
            let _ = kv_partition::set(&cache_key, payload.as_bytes().to_vec());

            Ok(ContextResponse {
                uri,
                level: resolved,
                token_estimate: super::estimate_tokens(&payload),
                payload,
            })
        }
    }

    export!(VikingContextImpl);
}

// ── Pure business logic (always compiled, tested on native) ──────────────────

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

/// Rough token estimate: 4 ASCII chars ≈ 1 token.
pub fn estimate_tokens(text: &str) -> u32 {
    u32::try_from(text.len() / 4).unwrap_or(u32::MAX)
}

/// L0 — one-paragraph summary suitable for bulk scanning by the agent.
pub fn build_summary(path: &str, content: &str) -> String {
    let line_count = content.lines().count();
    let mut out = format!("# {path}\nLines: {line_count}\n");

    if path.ends_with(".rs") && let Ok(file) = syn::parse_file(content) {
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

fn write_item_skeleton(out: &mut String, item: &syn::Item) {
    match item {
        syn::Item::Fn(f) => {
            let async_kw = if f.sig.asyncness.is_some() { "async " } else { "" };
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
                    let async_kw = if m.sig.asyncness.is_some() { "async " } else { "" };
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
                    let async_kw = if m.sig.asyncness.is_some() { "async " } else { "" };
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
}
