// ── WASM component glue ──────────────────────────────────────────────────────
// Only compiled when targeting wasm32-wasip2.  On native the pure-logic
// functions below are compiled and exercised by the unit-test suite.
#[cfg(target_arch = "wasm32")]
mod component {
    wit_bindgen::generate!({
        path: "wit",
        world: "viking-context-world",
    });

    use exports::tachyon::ai::viking_context::{ContextLevel, ContextResponse, Guest};

    struct VikingContextImpl;

    impl Guest for VikingContextImpl {
        fn resolve(uri: String, level: ContextLevel) -> Result<ContextResponse, String> {
            let path = super::strip_viking_prefix(&uri)?;
            let bytes = tachyon::ai::storage_broker::read_file(&path)?;
            let raw = String::from_utf8(bytes)
                .map_err(|_| format!("'{path}' is not valid UTF-8"))?;

            let (payload, resolved) = match level {
                ContextLevel::L2Raw => (raw, ContextLevel::L2Raw),
                ContextLevel::L1Structure => {
                    if path.ends_with(".rs") {
                        (super::extract_rust_skeleton(&raw), ContextLevel::L1Structure)
                    } else {
                        // Non-Rust files: fall back to raw content
                        (raw, ContextLevel::L2Raw)
                    }
                }
                ContextLevel::L0Summary => {
                    (super::build_summary(&path, &raw), ContextLevel::L0Summary)
                }
            };

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

// ── Pure business logic (always compiled) ────────────────────────────────────

/// Strip the `viking://` scheme prefix and return the file path.
pub fn strip_viking_prefix(uri: &str) -> Result<String, String> {
    uri.strip_prefix("viking://")
        .map(str::to_string)
        .ok_or_else(|| format!("invalid viking URI: expected 'viking://' prefix, got '{uri}'"))
}

/// Rough token estimate: 4 ASCII chars ≈ 1 token.
pub fn estimate_tokens(text: &str) -> u32 {
    u32::try_from(text.len() / 4).unwrap_or(u32::MAX)
}

/// L0 — one-paragraph summary suitable for bulk scanning.
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

/// L1 — AST skeleton for Rust files: signatures without bodies.
pub fn extract_rust_skeleton(src: &str) -> String {
    let Ok(file) = syn::parse_file(src) else {
        // If syn can't parse, fall back to raw source
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

        // use, extern crate, macro_rules!, etc. — omit to reduce noise
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

// ── Unit tests (native target only) ─────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn token_estimate_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn token_estimate_approx() {
        // 8 chars → 2 tokens
        assert_eq!(estimate_tokens("abcdefgh"), 2);
    }

    #[test]
    fn l0_summary_contains_path_and_lines() {
        let content = "fn main() {}\n";
        let summary = build_summary("src/main.rs", content);
        assert!(summary.contains("src/main.rs"));
        assert!(summary.contains("Lines: 1"));
    }

    #[test]
    fn l0_summary_lists_public_items() {
        let content = r#"
pub struct Config { pub name: String }
pub fn run() {}
fn private() {}
"#;
        let summary = build_summary("src/lib.rs", content);
        assert!(summary.contains("struct Config"));
        assert!(summary.contains("fn run"));
        assert!(!summary.contains("private"));
    }

    #[test]
    fn l1_skeleton_strips_bodies() {
        let src = r#"
pub struct Config {
    pub name: String,
    value: u32,
}

pub fn process(config: &Config) -> Result<(), String> {
    println!("{}", config.name);
    Ok(())
}

pub enum Status { Active, Inactive }

pub trait Handler {
    fn handle(&self, input: &str) -> String;
}

impl Handler for Config {
    fn handle(&self, input: &str) -> String {
        self.name.clone()
    }
}
"#;
        let skeleton = extract_rust_skeleton(src);

        // Structural markers present
        assert!(skeleton.contains("struct Config"));
        assert!(skeleton.contains("fn process"));
        assert!(skeleton.contains("enum Status"));
        assert!(skeleton.contains("trait Handler"));
        assert!(skeleton.contains("impl Handler for Config"));

        // Implementation details stripped
        assert!(!skeleton.contains("println!"));
        assert!(!skeleton.contains("name.clone()"));
    }

    #[test]
    fn l1_skeleton_fallback_on_non_rust_parse_error() {
        // A non-Rust string still falls back gracefully
        let raw = "not valid rust {{ {{ {";
        let result = extract_rust_skeleton(raw);
        // Fallback returns the original source unchanged
        assert_eq!(result, raw);
    }
}
