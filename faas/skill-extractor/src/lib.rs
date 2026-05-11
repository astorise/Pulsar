#[cfg(target_arch = "wasm32")]
mod component {
    wit_bindgen::generate!({
        path: "wit",
        world: "skill-extractor-world",
    });

    use exports::tachyon::ai::skill_extractor::{ExtractionRequest, ExtractionResponse, Guest};
    use tachyon::ai::{inference, storage_broker};

    struct SkillExtractor;

    impl Guest for SkillExtractor {
        fn extract(req: ExtractionRequest) -> Result<ExtractionResponse, String> {
            let prompt = super::build_meta_prompt(&req.task_description, &req.execution_trace);
            let inference_req = inference::InferenceRequest {
                model_id: "qwen-coder-27b".to_string(),
                prompt,
                max_tokens: 2048,
                temperature: 0.1,
                lora_adapter: None,
            };

            let response = inference::generate(&inference_req)?;
            let markdown = super::extract_markdown(&response.text);
            let filename = super::sanitize_task_name(&req.task_description);
            let skill_uri = format!(".pulsar/skills/{filename}.md");
            storage_broker::write_file(&skill_uri, markdown.as_bytes())?;

            Ok(ExtractionResponse {
                skill_uri,
                learned_concept: super::learned_concept(&markdown),
            })
        }
    }

    export!(SkillExtractor);
}

pub fn build_meta_prompt(task: &str, trace: &str) -> String {
    format!(
        r#"Analyze this successful agent execution trace and distill the reusable workflow.

Original task:
{task}

Execution trace JSON:
{trace}

Return only a Markdown document for SKILL.md with these exact sections:
# Skill
## Context / Trigger
Describe when this skill should be used.

## Steps
List the deterministic steps that solve the task. Avoid dead ends shown in the trace.

## Commands And Tools
List the specific commands, tools, files, or APIs to use.

Do not include commentary outside the Markdown document."#
    )
}

pub fn extract_markdown(response: &str) -> String {
    let trimmed = response.trim();
    if let Some(body) = trimmed
        .strip_prefix("```markdown")
        .or_else(|| trimmed.strip_prefix("```md"))
        .or_else(|| trimmed.strip_prefix("```"))
    {
        return body.trim_end_matches("```").trim().to_string();
    }

    trimmed.to_string()
}

pub fn sanitize_task_name(task: &str) -> String {
    let mut out = String::new();
    let mut previous_dash = false;

    for ch in task.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            previous_dash = false;
        } else if !previous_dash && !out.is_empty() {
            out.push('-');
            previous_dash = true;
        }
    }

    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "generated-skill".to_string()
    } else {
        trimmed
            .chars()
            .take(80)
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
}

pub fn learned_concept(markdown: &str) -> String {
    markdown
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("# ")
                .or_else(|| line.strip_prefix("## "))
                .map(str::trim)
                .filter(|title| !title.eq_ignore_ascii_case("skill"))
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Reusable workflow extracted from execution trace".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_prompt_requires_strict_skill_sections() {
        let prompt = build_meta_prompt("refactor parser", r#"{"steps":[]}"#);

        assert!(prompt.contains("Original task:\nrefactor parser"));
        assert!(prompt.contains("Execution trace JSON:\n{\"steps\":[]}"));
        assert!(prompt.contains("## Context / Trigger"));
        assert!(prompt.contains("## Commands And Tools"));
        assert!(prompt.contains("Do not include commentary outside the Markdown document."));
    }

    #[test]
    fn markdown_fence_is_removed() {
        let markdown = extract_markdown("```markdown\n# Skill\nbody\n```");

        assert_eq!(markdown, "# Skill\nbody");
    }

    #[test]
    fn task_name_is_filesystem_safe() {
        assert_eq!(
            sanitize_task_name("Fix AST parser: lifetimes & tests!"),
            "fix-ast-parser-lifetimes-tests"
        );
    }

    #[test]
    fn empty_task_name_gets_stable_fallback() {
        assert_eq!(sanitize_task_name("!!!"), "generated-skill");
    }

    #[test]
    fn learned_concept_prefers_specific_heading() {
        let markdown = "# Skill\n## Refactor parser safely\nBody";

        assert_eq!(learned_concept(markdown), "Refactor parser safely");
    }
}
