use serde::{Deserialize, Serialize};

pub const TECH_LEAD_PROMPT: &str = "You are the Tech Lead. Do not write code. Break down the user's request into independent files/components to be worked on in parallel.";
pub const DEFAULT_TOKEN_BUDGET: u64 = 120_000;
pub const SKILL_THRESHOLD: f32 = 0.80;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub files: Vec<String>,
    pub component: String,
    pub skill_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChildStatus {
    Running,
    Failed,
    Succeeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildJob {
    pub branch: String,
    pub status: ChildStatus,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillDef {
    pub name: String,
    pub description_embedding: Vec<f32>,
    pub system_prompt: String,
    pub allowed_tools: Vec<String>,
}

pub fn build_map_prompt(user_task: &str) -> String {
    format!(
        "{TECH_LEAD_PROMPT}\n\nReturn JSON array of subtasks with id, title, component, and files.\n\nTask:\n{user_task}"
    )
}

pub fn parse_subtasks_json(json: &str) -> Result<Vec<SubTask>, String> {
    serde_json::from_str(json).map_err(|err| err.to_string())
}

pub fn child_env(skill_name: Option<&str>) -> Vec<(String, String)> {
    skill_name
        .map(|name| vec![("PULSAR_SKILL_NAME".to_string(), name.to_string())])
        .unwrap_or_default()
}

pub fn merge_order(children: &[ChildJob]) -> Vec<String> {
    children
        .iter()
        .filter(|child| matches!(child.status, ChildStatus::Succeeded))
        .map(|child| child.branch.clone())
        .collect()
}

pub fn extract_conflict_blocks(file: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut in_conflict = false;

    for line in file.lines() {
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            current.clear();
        }
        if in_conflict {
            current.push(line);
        }
        if line.starts_with(">>>>>>>") && in_conflict {
            blocks.push(current.join("\n"));
            in_conflict = false;
        }
    }

    blocks
}

pub fn should_spawn_browser_agent(files: &[String]) -> bool {
    files.iter().any(|file| {
        matches!(
            file.rsplit('.').next(),
            Some("html" | "css" | "js" | "jsx" | "ts" | "tsx")
        )
    })
}

pub fn token_budget_exceeded(current: u64, next: u64, budget: u64) -> bool {
    current.saturating_add(next) > budget
}

pub fn select_skill<'a>(
    task_embedding: &[f32],
    skills: &'a [SkillDef],
    threshold: f32,
) -> Option<&'a SkillDef> {
    skills
        .iter()
        .filter_map(|skill| {
            let score = cosine_similarity(task_embedding, &skill.description_embedding);
            (score >= threshold).then_some((score, skill))
        })
        .max_by(|(left, _), (right, _)| left.total_cmp(right))
        .map(|(_, skill)| skill)
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.is_empty() || left.len() != right.len() {
        return 0.0;
    }
    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(l, r)| l * r)
        .sum::<f32>();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_prompt_uses_tech_lead_role() {
        let prompt = build_map_prompt("build UI and API");
        assert!(prompt.contains("You are the Tech Lead"));
        assert!(prompt.contains("build UI and API"));
    }

    #[test]
    fn subtasks_parse_from_json() {
        let tasks = parse_subtasks_json(
            r#"[{"id":"1","title":"UI","files":["src/App.tsx"],"component":"frontend","skill_name":null}]"#,
        )
        .unwrap();
        assert_eq!(tasks[0].files, vec!["src/App.tsx"]);
    }

    #[test]
    fn reduce_merges_only_successful_children() {
        let branches = merge_order(&[
            ChildJob {
                branch: "a".to_string(),
                status: ChildStatus::Succeeded,
                files: vec![],
            },
            ChildJob {
                branch: "b".to_string(),
                status: ChildStatus::Failed,
                files: vec![],
            },
        ]);
        assert_eq!(branches, vec!["a"]);
    }

    #[test]
    fn conflict_blocks_are_extracted() {
        let blocks =
            extract_conflict_blocks("a\n<<<<<<< HEAD\nleft\n=======\nright\n>>>>>>> branch\nb");
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("left"));
    }

    #[test]
    fn ui_files_trigger_browser_agent() {
        assert!(should_spawn_browser_agent(&["web/app.css".to_string()]));
        assert!(!should_spawn_browser_agent(&["src/lib.rs".to_string()]));
    }

    #[test]
    fn semantic_router_selects_best_skill() {
        let skills = vec![SkillDef {
            name: "ui".to_string(),
            description_embedding: vec![1.0, 0.0],
            system_prompt: "prompt".to_string(),
            allowed_tools: vec![],
        }];
        assert_eq!(
            select_skill(&[1.0, 0.0], &skills, SKILL_THRESHOLD)
                .unwrap()
                .name,
            "ui"
        );
    }
}
