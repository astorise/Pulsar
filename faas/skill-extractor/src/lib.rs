#[cfg(target_arch = "wasm32")]
mod component {
    wit_bindgen::generate!({
        path: "wit",
        world: "skill-extractor-world",
    });

    use exports::tachyon::ai::skill_extractor::{ExtractionRequest, ExtractionResponse, Guest};
    use tachyon::ai::{inference, storage_broker, training};

    struct SkillExtractor;

    impl Guest for SkillExtractor {
        fn extract(req: ExtractionRequest) -> Result<ExtractionResponse, String> {
            let filtered_trace = super::filter_success_trace(&req.execution_trace);
            let prompt = super::build_meta_prompt(&req.task_description, &filtered_trace);
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

            let dataset_line = super::sharegpt_jsonl(&req.task_description, &filtered_trace)?;
            storage_broker::append_file(super::DATASET_PATH, dataset_line.as_bytes())?;
            if super::should_submit_training(
                super::generated_example_count(&dataset_line),
                super::TRAINING_THRESHOLD,
            ) {
                let _ = training::submit_job(super::DATASET_PATH, "qwen-coder-27b");
            }

            Ok(ExtractionResponse {
                skill_uri,
                learned_concept: super::learned_concept(&markdown),
            })
        }
    }

    export!(SkillExtractor);
}

use serde::{Deserialize, Serialize};

pub const DATASET_PATH: &str = ".pulsar/kiln/sharegpt.jsonl";
pub const TRAINING_THRESHOLD: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceItem {
    pub phase: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    pub conversations: Vec<Message>,
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

pub fn filter_success_trace(trace: &str) -> String {
    let Ok(events) = serde_json::from_str::<Vec<TraceItem>>(trace) else {
        return trace.to_string();
    };

    let filtered = events
        .into_iter()
        .filter(|event| event.phase != "run_command" || command_output_succeeded(&event.content))
        .collect::<Vec<_>>();
    serde_json::to_string(&filtered).unwrap_or_else(|_| trace.to_string())
}

pub fn sharegpt_jsonl(task: &str, filtered_trace: &str) -> Result<String, String> {
    let conversation = Conversation {
        conversations: vec![
            Message {
                from: "human".to_string(),
                value: task.to_string(),
            },
            Message {
                from: "gpt".to_string(),
                value: filtered_trace.to_string(),
            },
        ],
    };
    serde_json::to_string(&conversation)
        .map(|line| format!("{line}\n"))
        .map_err(|err| err.to_string())
}

pub fn should_submit_training(dataset_size: usize, threshold: usize) -> bool {
    threshold > 0 && dataset_size >= threshold && dataset_size.is_multiple_of(threshold)
}

pub fn generated_example_count(jsonl: &str) -> usize {
    jsonl.lines().filter(|line| !line.trim().is_empty()).count()
}

pub fn telemetry_message(dataset_size: usize, training_submitted: bool) -> String {
    if training_submitted {
        format!(
            "[Kiln] New training example generated. Dataset size: {dataset_size}. LoRA training job submitted to Tachyon host."
        )
    } else {
        format!("[Kiln] New training example generated. Dataset size: {dataset_size}.")
    }
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

fn command_output_succeeded(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    !(lower.contains("error:")
        || lower.contains("failed")
        || lower.contains("panicked")
        || lower.contains("exit code")
        || lower.contains("traceback"))
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

    #[test]
    fn trace_filter_removes_failed_command_output() {
        let trace = serde_json::to_string(&vec![
            TraceItem {
                phase: "prompt".to_string(),
                content: "fix tests".to_string(),
            },
            TraceItem {
                phase: "run_command".to_string(),
                content: "error: failed".to_string(),
            },
            TraceItem {
                phase: "run_command".to_string(),
                content: "test result: ok".to_string(),
            },
        ])
        .unwrap();

        let filtered = filter_success_trace(&trace);
        assert!(!filtered.contains("error: failed"));
        assert!(filtered.contains("test result: ok"));
    }

    #[test]
    fn sharegpt_formatter_emits_jsonl() {
        let line = sharegpt_jsonl("fix tests", r#"[{"phase":"finish"}]"#).unwrap();
        assert!(line.ends_with('\n'));
        assert!(line.contains("\"from\":\"human\""));
        assert!(line.contains("\"from\":\"gpt\""));
    }

    #[test]
    fn training_threshold_is_periodic() {
        assert!(should_submit_training(500, 500));
        assert!(should_submit_training(1000, 500));
        assert!(!should_submit_training(499, 500));
    }

    #[test]
    fn telemetry_mentions_training_when_submitted() {
        let message = telemetry_message(500, true);
        assert!(message.contains("Dataset size: 500"));
        assert!(message.contains("LoRA training job submitted"));
    }
}
