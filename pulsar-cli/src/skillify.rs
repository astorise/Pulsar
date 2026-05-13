use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillDef {
    pub name: String,
    pub description_embedding: Vec<f32>,
    pub system_prompt: String,
    pub allowed_tools: Vec<String>,
}

pub fn run(root: &Path) -> anyhow::Result<String> {
    let skills_dir = root.join(".pulsar").join("skills");
    let skills = discover_skill_files(&skills_dir)?;
    let mut compiled = Vec::new();
    for path in &skills {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        compiled.push(compile_skill(path, &content)?);
    }

    let registry = root
        .join(".pulsar")
        .join("skill-registry")
        .join("pulsar_skill_registry.msgpack");
    if let Some(parent) = registry.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let bytes = rmp_serde::to_vec_named(&compiled).context("failed to encode skill registry")?;
    fs::write(&registry, bytes)
        .with_context(|| format!("failed to write {}", registry.display()))?;

    Ok(format!(
        "Compiled {} skills into {}",
        compiled.len(),
        registry.display()
    ))
}

pub fn discover_skill_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    visit_skill_dir(root, &mut out)?;
    out.sort();
    Ok(out)
}

pub fn compile_skill(path: &Path, content: &str) -> anyhow::Result<SkillDef> {
    let (frontmatter, body) = split_frontmatter(content);
    let fallback_name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("skill")
        .to_string();
    let name = frontmatter.get("name").cloned().unwrap_or(fallback_name);
    let description = frontmatter
        .get("description")
        .cloned()
        .unwrap_or_else(|| first_heading(body).unwrap_or_else(|| name.clone()));
    let allowed_tools = frontmatter
        .get("allowed_tools")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(SkillDef {
        name,
        description_embedding: embed_description(&description),
        system_prompt: body.trim().to_string(),
        allowed_tools,
    })
}

pub fn split_frontmatter(content: &str) -> (BTreeMap<String, String>, &str) {
    let mut map = BTreeMap::new();
    let Some(rest) = content.strip_prefix("---\n") else {
        return (map, content);
    };
    let Some((frontmatter, body)) = rest.split_once("\n---\n") else {
        return (map, content);
    };

    for line in frontmatter.lines() {
        if let Some((key, value)) = line.split_once(':') {
            map.insert(
                key.trim().to_string(),
                value.trim().trim_matches('"').to_string(),
            );
        }
    }
    (map, body)
}

pub fn embed_description(description: &str) -> Vec<f32> {
    let mut vector = vec![0.0_f32; 16];
    for token in description
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        let mut hash = 2166136261_u32;
        for byte in token.bytes() {
            hash ^= u32::from(byte.to_ascii_lowercase());
            hash = hash.wrapping_mul(16777619);
        }
        let index = (hash as usize) % vector.len();
        vector[index] += 1.0;
    }
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }
    vector
}

fn visit_skill_dir(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_skill_dir(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

fn first_heading(content: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(str::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_is_split_from_body() {
        let (frontmatter, body) =
            split_frontmatter("---\nname: Rust Fixer\nallowed_tools: search, edit\n---\n# Body");

        assert_eq!(frontmatter.get("name").unwrap(), "Rust Fixer");
        assert_eq!(body.trim(), "# Body");
    }

    #[test]
    fn skill_compiles_to_msgpack_safe_struct() {
        let skill = compile_skill(
            Path::new("SKILL.md"),
            "---\nname: Rust Fixer\ndescription: Fix Rust tests\nallowed_tools: search, edit\n---\n# Prompt",
        )
        .unwrap();
        let encoded = rmp_serde::to_vec_named(&skill).unwrap();

        assert_eq!(skill.name, "Rust Fixer");
        assert_eq!(skill.allowed_tools, vec!["search", "edit"]);
        assert!(!encoded.is_empty());
    }
}
