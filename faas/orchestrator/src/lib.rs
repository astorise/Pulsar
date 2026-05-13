mod sanitizer;
pub use sanitizer::{clean_output, smart_truncate};

#[cfg(target_arch = "wasm32")]
mod component {
    wit_bindgen::generate!({
        path: "wit",
        world: "orchestrator-world",
    });

    use exports::tachyon::ai::orchestrator::{Guest, SessionConfig};
    use std::sync::{LazyLock, Mutex};
    use tachyon::ai::{inference, kv_partition, skill_extractor, viking_context, workspace_bridge};

    static SESSIONS: LazyLock<Mutex<Vec<super::Session>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));

    struct Orchestrator;

    impl Guest for Orchestrator {
        fn start_session(cfg: SessionConfig) -> Result<String, String> {
            let config = super::SessionConfig {
                workspace_url: cfg.workspace_url,
                workspace_token: cfg.workspace_token,
                max_tier: cfg.max_tier,
            };
            let session = super::Session::new(config)?;
            let session_id = session.id.clone();
            SESSIONS
                .lock()
                .map_err(|_| "session store is poisoned".to_string())?
                .push(session);
            let _ = crawl_workspace(&session_id);
            Ok(session_id)
        }

        fn submit_input(session_id: String, text: String) -> Result<(), String> {
            let mut sessions = SESSIONS
                .lock()
                .map_err(|_| "session store is poisoned".to_string())?;
            let session = sessions
                .iter_mut()
                .find(|session| session.id == session_id)
                .ok_or_else(|| format!("unknown session: {session_id}"))?;

            session.record_user_input(&text);
            session.status = super::AgentStatus::Recalling;
            let past_skill = discover_skill(&session.id, &text);

            session.status = super::AgentStatus::Thinking;
            let context_uri = super::viking_uri_for_prompt(&text);
            let context =
                viking_context::resolve(&context_uri, viking_context::ContextLevel::L1Structure)
                    .or_else(|_| {
                        viking_context::resolve(
                            &context_uri,
                            viking_context::ContextLevel::L0Summary,
                        )
                    })?;
            session.record_observation("viking-context", &context.payload);

            let swarm_intel = fetch_recent_intel().unwrap_or_default();
            let approved_plan = session.approved_plan.as_ref().map(super::Plan::pinned_text);
            let prompt = super::build_inference_prompt(
                &session.trace,
                &context.payload,
                &text,
                past_skill.as_deref(),
                non_empty(&swarm_intel),
                approved_plan.as_deref(),
            );
            let response = inference::generate(&inference::InferenceRequest {
                model_id: super::model_for_tier(session.config.max_tier).to_string(),
                prompt,
                max_tokens: 1024,
                temperature: 0.1,
                lora_adapter: session.active_skill.clone(),
            })?;
            session.record_token_usage(response.prompt_tokens + response.completion_tokens);
            increment_token_counter(
                &session.id,
                response.prompt_tokens + response.completion_tokens,
            );

            session.status = super::AgentStatus::Acting;
            let tool_call = super::parse_tool_call(&response.text)?;
            execute_tool_call(session, &text, tool_call)?;

            if !matches!(
                session.status,
                super::AgentStatus::Finished
                    | super::AgentStatus::Suspended
                    | super::AgentStatus::Escalated
            ) {
                session.status = super::AgentStatus::AwaitingUser;
            }

            Ok(())
        }

        fn resume_session(session_id: String, human_feedback: String) -> Result<(), String> {
            let key = super::suspended_session_key(&session_id);
            let bytes = workspace_bridge::kv_get(&key)?
                .ok_or_else(|| format!("no suspended session found for {session_id}"))?;
            let mut session: super::Session =
                serde_json::from_slice(&bytes).map_err(|err| err.to_string())?;
            session.record_observation("human_feedback", &human_feedback);
            session.status = super::AgentStatus::AwaitingUser;

            let mut sessions = SESSIONS
                .lock()
                .map_err(|_| "session store is poisoned".to_string())?;
            if let Some(existing) = sessions.iter_mut().find(|item| item.id == session_id) {
                *existing = session;
            } else {
                sessions.push(session);
            }
            Ok(())
        }
    }

    fn execute_tool_call(
        session: &mut super::Session,
        original_task: &str,
        tool_call: super::ToolCall,
    ) -> Result<(), String> {
        match tool_call {
            super::ToolCall::SubmitPlan {
                goal,
                branch,
                files,
                steps,
            } => {
                let plan = super::Plan {
                    goal,
                    branch,
                    files,
                    steps,
                };
                session.approved_plan = Some(plan.clone());
                session.status = super::AgentStatus::Suspended;
                session.record_observation("submit_plan", &plan.pinned_text());
                suspend_session(session)?;
                emit_json(
                    &session.id,
                    serde_json::json!({"type":"handshake","plan":plan}),
                );
            }
            super::ToolCall::SearchVikingContext { query } => {
                let matches = viking_context::search(&query)?;
                session.record_observation("search_viking_context", &matches.join("\n"));
            }
            super::ToolCall::ReadVikingContext { uri } => {
                let resolved =
                    viking_context::resolve(&uri, viking_context::ContextLevel::L1Structure)?;
                session.record_observation("read_viking_context", &resolved.payload);
            }
            super::ToolCall::QueryGraph { entity, depth } => {
                let results = viking_context::graph_query(&entity, depth)?;
                session.record_observation("query_graph", &results.join("\n"));
            }
            super::ToolCall::AskLsp {
                path,
                line,
                character,
            } => {
                let hover = workspace_bridge::lsp_hover(&session.id, &path, line, character)?;
                session.record_observation("ask_lsp", &hover);
            }
            super::ToolCall::EditFile { path, contents } => {
                if !session.has_approved_plan() {
                    session
                        .record_observation("planning_required", "You must use submit_plan first");
                    return Ok(());
                }
                workspace_bridge::webdav_put(&session.id, &path, contents.as_bytes())?;
                session.record_observation("edit_file", &format!("updated {path}"));
                let _ = broadcast_to_swarm(session, &format!("updated {path}"));
            }
            super::ToolCall::RunCommand { command } => {
                if !session.has_approved_plan() {
                    session
                        .record_observation("planning_required", "You must use submit_plan first");
                    return Ok(());
                }
                let raw = workspace_bridge::websocket_command(&session.id, &command)
                    .unwrap_or_else(|err| format!("ERROR: {err}"));
                let cleaned = super::clean_output(&command, &raw);
                session.record_command_result(&command, &cleaned);
                session.record_observation("run_command", &cleaned);
                let _ = broadcast_to_swarm(session, &cleaned);
                if session.consecutive_failures >= super::FAILURE_ESCALATION_THRESHOLD {
                    let report_prompt =
                        super::build_situation_report_prompt(original_task, &session.trace);
                    let report = inference::generate(&inference::InferenceRequest {
                        model_id: super::model_for_tier(session.config.max_tier).to_string(),
                        prompt: report_prompt,
                        max_tokens: 768,
                        temperature: 0.1,
                        lora_adapter: None,
                    })
                    .map(|response| response.text)
                    .unwrap_or_else(|_| super::fallback_situation_report(original_task, session));
                    session.status = super::AgentStatus::Escalated;
                    session.record_observation("situation_report", &report);
                    emit_json(
                        &session.id,
                        serde_json::json!({"type":"escalated","report":report}),
                    );
                }
            }
            super::ToolCall::RequestHumanAction {
                instruction,
                requires_feedback,
            } => {
                session.status = super::AgentStatus::Suspended;
                session.record_observation("request_human_action", &instruction);
                suspend_session(session)?;
                emit_json(
                    &session.id,
                    serde_json::json!({
                        "type":"suspend",
                        "instruction":instruction,
                        "requires_feedback":requires_feedback
                    }),
                );
            }
            super::ToolCall::Finish { summary } => {
                session.status = super::AgentStatus::Finished;
                session.record_observation("finish", &summary);
                emit_json(
                    &session.id,
                    serde_json::json!({"type":"action_event","action":"finish","target":"session"}),
                );
                let trace_json = session.trace_json()?;
                let _ = skill_extractor::extract(&skill_extractor::ExtractionRequest {
                    task_description: original_task.to_string(),
                    execution_trace: trace_json,
                });
            }
        }

        Ok(())
    }

    fn crawl_workspace(session_id: &str) -> Result<(), String> {
        let entries = workspace_bridge::webdav_propfind(session_id, ".")?;
        for entry in entries
            .iter()
            .filter(|entry| super::is_indexable_workspace_path(entry))
        {
            let uri = format!("viking://{entry}");
            let _ = viking_context::resolve(&uri, viking_context::ContextLevel::L0Summary);
        }
        Ok(())
    }

    fn discover_skill(session_id: &str, prompt: &str) -> Option<String> {
        let skills = workspace_bridge::webdav_propfind(session_id, ".pulsar/skills").ok()?;
        let selected = super::match_skill(prompt, &skills)?;
        let path = if selected.starts_with(".pulsar/skills/") {
            selected
        } else {
            format!(".pulsar/skills/{selected}")
        };
        let bytes = workspace_bridge::webdav_get(session_id, &path).ok()?;
        String::from_utf8(bytes).ok()
    }

    fn suspend_session(session: &super::Session) -> Result<(), String> {
        let key = super::suspended_session_key(&session.id);
        let bytes = serde_json::to_vec(session).map_err(|err| err.to_string())?;
        workspace_bridge::kv_set(&key, &bytes)
    }

    fn fetch_recent_intel() -> Result<String, String> {
        let table = kv_partition::Table::new("mcp_observations");
        let rows = table.get_range("v1:mcp:", 0, 50)?;
        let observations = rows
            .into_iter()
            .filter_map(|(_key, bytes)| {
                serde_json::from_slice::<super::McpObservation>(&bytes).ok()
            })
            .collect::<Vec<_>>();
        Ok(super::format_recent_intel(&observations, 8000))
    }

    fn broadcast_to_swarm(session: &super::Session, content: &str) -> Result<(), String> {
        let timestamp = session
            .trace
            .last()
            .map(|_| session.trace.len() as u64)
            .unwrap_or_default();
        let observation = super::McpObservation {
            author_session: session.id.clone(),
            timestamp,
            related_files: Vec::new(),
            content: content.to_string(),
        };
        let table = kv_partition::Table::new("mcp_observations");
        let key = super::mcp_observation_key(&observation.author_session, observation.timestamp);
        let bytes = serde_json::to_vec(&observation).map_err(|err| err.to_string())?;
        table.set(&key, &bytes)
    }

    fn increment_token_counter(session_id: &str, tokens: u32) {
        let table = kv_partition::Table::new("mcp_observations");
        let key = format!("v1:metrics:swarm_tokens:{session_id}");
        let current = table
            .get(&key)
            .ok()
            .flatten()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| text.parse::<u64>().ok())
            .unwrap_or_default();
        let next = current.saturating_add(u64::from(tokens));
        let _ = table.set(&key, next.to_string().as_bytes());
    }

    fn emit_json(session_id: &str, payload: serde_json::Value) {
        let _ = workspace_bridge::emit_event(session_id, &payload.to_string());
    }

    fn non_empty(value: &str) -> Option<&str> {
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    }

    export!(Orchestrator);
}

use serde::{Deserialize, Serialize};

pub const FAILURE_ESCALATION_THRESHOLD: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConfig {
    pub workspace_url: String,
    pub workspace_token: String,
    pub max_tier: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Planning,
    Recalling,
    Thinking,
    Acting,
    AwaitingUser,
    Suspended,
    Escalated,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub phase: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub goal: String,
    pub branch: Option<String>,
    pub files: Vec<String>,
    pub steps: Vec<String>,
}

impl Plan {
    pub fn pinned_text(&self) -> String {
        format!(
            "Goal: {}\nBranch: {}\nFiles: {}\nSteps:\n{}",
            self.goal,
            self.branch.as_deref().unwrap_or("current"),
            if self.files.is_empty() {
                "unspecified".to_string()
            } else {
                self.files.join(", ")
            },
            self.steps
                .iter()
                .map(|step| format!("- {step}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub config: SessionConfig,
    pub status: AgentStatus,
    pub trace: Vec<TraceEvent>,
    pub consecutive_failures: u32,
    pub last_failed_target: Option<String>,
    pub approved_plan: Option<Plan>,
    pub active_skill: Option<String>,
    pub token_usage: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum ToolCall {
    SubmitPlan {
        goal: String,
        branch: Option<String>,
        files: Vec<String>,
        steps: Vec<String>,
    },
    SearchVikingContext {
        query: String,
    },
    ReadVikingContext {
        uri: String,
    },
    QueryGraph {
        entity: String,
        depth: u32,
    },
    AskLsp {
        path: String,
        line: u32,
        character: u32,
    },
    EditFile {
        path: String,
        contents: String,
    },
    RunCommand {
        command: String,
    },
    RequestHumanAction {
        instruction: String,
        #[serde(alias = "requires_text_feedback")]
        requires_feedback: bool,
    },
    Finish {
        summary: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpObservation {
    pub author_session: String,
    pub timestamp: u64,
    pub related_files: Vec<String>,
    pub content: String,
}

impl Session {
    pub fn new(config: SessionConfig) -> Result<Self, String> {
        validate_session_config(&config)?;
        Ok(Self {
            id: build_session_id(&config),
            config,
            status: AgentStatus::Planning,
            trace: Vec::new(),
            consecutive_failures: 0,
            last_failed_target: None,
            approved_plan: None,
            active_skill: None,
            token_usage: 0,
        })
    }

    pub fn has_approved_plan(&self) -> bool {
        self.approved_plan.is_some()
    }

    pub fn record_user_input(&mut self, input: &str) {
        self.trace.push(TraceEvent {
            phase: "prompt".to_string(),
            content: input.to_string(),
        });
    }

    pub fn record_observation(&mut self, phase: &str, content: &str) {
        self.trace.push(TraceEvent {
            phase: phase.to_string(),
            content: content.to_string(),
        });
    }

    pub fn record_command_result(&mut self, command: &str, output: &str) {
        let target = command_failure_target(command);
        if command_output_succeeded(output) {
            self.consecutive_failures = 0;
            self.last_failed_target = None;
            return;
        }

        if self.last_failed_target.as_deref() == Some(target.as_str()) {
            self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        } else {
            self.last_failed_target = Some(target);
            self.consecutive_failures = 1;
        }
    }

    pub fn record_token_usage(&mut self, tokens: u32) {
        self.token_usage = self.token_usage.saturating_add(u64::from(tokens));
    }

    pub fn trace_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.trace).map_err(|err| err.to_string())
    }
}

pub fn validate_session_config(config: &SessionConfig) -> Result<(), String> {
    if normalize_workspace_url(&config.workspace_url)?.is_empty() {
        return Err("workspace-url must not be empty".to_string());
    }
    if config.workspace_token.trim().is_empty() {
        return Err("workspace-token must not be empty".to_string());
    }
    if config.max_tier == 0 {
        return Err("max-tier must be at least 1".to_string());
    }
    Ok(())
}

pub fn normalize_workspace_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("workspace-url must not be empty".to_string());
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("workspace-url must start with http:// or https://".to_string());
    }
    Ok(trimmed.to_string())
}

pub fn build_session_id(config: &SessionConfig) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in config
        .workspace_url
        .bytes()
        .chain(config.workspace_token.bytes())
        .chain(config.max_tier.to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("sess-{hash:016x}")
}

pub fn webdav_path(workspace_url: &str, path: &str) -> Result<String, String> {
    let base = normalize_workspace_url(workspace_url)?;
    let path = path.trim_start_matches('/');
    Ok(format!("{base}/{path}"))
}

pub fn propfind_body(depth: u32) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:displayname/>
    <d:getcontentlength/>
    <d:getlastmodified/>
  </d:prop>
  <d:depth>{depth}</d:depth>
</d:propfind>"#
    )
}

pub fn build_inference_prompt(
    trace: &[TraceEvent],
    context: &str,
    user_input: &str,
    past_skill: Option<&str>,
    swarm_intel: Option<&str>,
    approved_plan: Option<&str>,
) -> String {
    let trace_json = serde_json::to_string(trace).unwrap_or_else(|_| "[]".to_string());
    let skill_section = past_skill
        .map(|skill| format!("\n--- PAST EXPERIENCE (SKILL) ---\n{skill}\n"))
        .unwrap_or_default();
    let swarm_section = swarm_intel
        .map(|intel| format!("\n### RECENT SWARM INTELLIGENCE\n{intel}\n"))
        .unwrap_or_default();
    let plan_section = approved_plan
        .map(|plan| format!("\n[APPROVED PLAN]\n{plan}\n"))
        .unwrap_or_default();

    format!(
        r#"You are the Pulsar orchestrator. Decide the next tool call.

Before modifying code, ALWAYS use `search_viking_context` to find relevant files, then `read_viking_context` to understand their structure. Do not waste tokens listing directories manually.
If you encounter an external function or type and are unsure of its signature or parameters, DO NOT guess. Use the `ask_lsp` tool to hover over it and read the exact documentation from the compiler.
If you need to verify a physical device state, read an email, or check visual aesthetics, DO NOT guess. Use `request_human_action` to ask the developer. You will be suspended and woken up when the task is done.
If no approved plan is present, use `submit_plan` before command execution or file edits.
{plan_section}{skill_section}{swarm_section}
User input:
{user_input}

Semantic context:
{context}

Execution trace:
{trace_json}

Return exactly one JSON object using one of these forms:
{{"tool":"submit_plan","goal":"goal","branch":"branch-name","files":["repo/path"],"steps":["step"]}}
{{"tool":"search_viking_context","query":"keywords"}}
{{"tool":"read_viking_context","uri":"viking://path"}}
{{"tool":"query_graph","entity":"struct:module::Type","depth":2}}
{{"tool":"ask_lsp","path":"src/lib.rs","line":1,"character":1}}
{{"tool":"edit_file","path":"repo/path","contents":"new file contents"}}
{{"tool":"run_command","command":"cargo test"}}
{{"tool":"request_human_action","instruction":"check the rendered UI","requires_feedback":true}}
{{"tool":"finish","summary":"done"}}"#
    )
}

pub fn parse_tool_call(text: &str) -> Result<ToolCall, String> {
    let trimmed = text.trim();
    let json = if let Some(start) = trimmed.find('{') {
        let end = trimmed
            .rfind('}')
            .ok_or_else(|| "tool call JSON is missing closing brace".to_string())?;
        &trimmed[start..=end]
    } else {
        trimmed
    };

    serde_json::from_str(json).map_err(|err| format!("invalid tool call JSON: {err}"))
}

pub fn viking_uri_for_prompt(prompt: &str) -> String {
    prompt
        .split_whitespace()
        .find(|word| word.contains('/') || word.ends_with(".rs") || word.ends_with(".toml"))
        .map(|word| {
            format!(
                "viking://{}",
                word.trim_matches(|ch| ch == '`' || ch == ',')
            )
        })
        .unwrap_or_else(|| "viking://.".to_string())
}

pub fn model_for_tier(max_tier: u32) -> &'static str {
    if max_tier >= 2 {
        "qwen-coder-27b"
    } else {
        "qwen-coder-7b"
    }
}

pub fn learning_trace_payload(task: &str, trace: &[TraceEvent]) -> Result<String, String> {
    let payload = serde_json::json!({
        "task_description": task,
        "execution_trace": trace,
    });
    serde_json::to_string(&payload).map_err(|err| err.to_string())
}

pub fn match_skill(prompt: &str, available_skills: &[String]) -> Option<String> {
    let prompt_tokens = tokenize(prompt);
    available_skills
        .iter()
        .filter_map(|skill| {
            let skill_tokens = tokenize(skill);
            let score = prompt_tokens
                .iter()
                .filter(|token| skill_tokens.contains(token))
                .count();
            (score > 0).then_some((score, skill))
        })
        .max_by_key(|(score, skill)| (*score, std::cmp::Reverse(skill.len())))
        .map(|(_, skill)| skill.clone())
}

pub fn is_indexable_workspace_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    if normalized.split('/').any(|part| {
        part.is_empty()
            || part.starts_with('.')
            || matches!(part, "node_modules" | "target" | ".git")
    }) {
        return false;
    }

    matches!(
        normalized.rsplit('.').next(),
        Some("rs" | "toml" | "md" | "json" | "ts" | "tsx" | "js" | "jsx" | "css" | "html")
    )
}

pub fn command_output_succeeded(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    !(lower.contains("error:")
        || lower.contains("failed")
        || lower.contains("panicked")
        || lower.contains("exit code")
        || lower.contains("traceback"))
}

pub fn command_failure_target(command: &str) -> String {
    command
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn build_situation_report_prompt(task: &str, trace: &[TraceEvent]) -> String {
    let trace_json = serde_json::to_string(trace).unwrap_or_else(|_| "[]".to_string());
    format!(
        r#"Write a Markdown situation report for a developer handoff.

Use contextual affirmation: acknowledge the concrete task and current evidence.
Use crisis handoff: state what failed, what was tried, and what decision is needed next.
Do not blame the developer or invent facts.

Task:
{task}

Trace:
{trace_json}"#
    )
}

pub fn fallback_situation_report(task: &str, session: &Session) -> String {
    format!(
        "# Situation Report\n\nTask: {task}\n\nRepeated failure target: {}\n\nConsecutive failures: {}\n",
        session.last_failed_target.as_deref().unwrap_or("unknown"),
        session.consecutive_failures
    )
}

pub fn suspended_session_key(session_id: &str) -> String {
    format!("v1:sessions:suspended:{session_id}")
}

pub fn sortable_timestamp_key(timestamp: u64) -> String {
    format!("{timestamp:016x}")
}

pub fn mcp_observation_key(session_id: &str, timestamp: u64) -> String {
    format!("v1:mcp:{}:{session_id}", sortable_timestamp_key(timestamp))
}

pub fn format_recent_intel(observations: &[McpObservation], max_chars: usize) -> String {
    let mut out = String::new();
    for observation in observations {
        let line = format!(
            "- [{}] {}: {}\n",
            observation.timestamp, observation.author_session, observation.content
        );
        if out.len().saturating_add(line.len()) > max_chars {
            break;
        }
        out.push_str(&line);
    }
    out
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() > 2)
        .map(str::to_ascii_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> SessionConfig {
        SessionConfig {
            workspace_url: "http://127.0.0.1:49152/workspace/".to_string(),
            workspace_token: "secret".to_string(),
            max_tier: 2,
        }
    }

    #[test]
    fn session_config_validates_url_token_and_tier() {
        assert!(validate_session_config(&config()).is_ok());

        let mut invalid = config();
        invalid.max_tier = 0;
        assert_eq!(
            validate_session_config(&invalid).unwrap_err(),
            "max-tier must be at least 1"
        );
    }

    #[test]
    fn webdav_paths_are_joined_without_double_slash() {
        assert_eq!(
            webdav_path("http://localhost:8080/root/", "/src/main.rs").unwrap(),
            "http://localhost:8080/root/src/main.rs"
        );
    }

    #[test]
    fn propfind_body_contains_requested_depth() {
        assert!(propfind_body(1).contains("<d:depth>1</d:depth>"));
    }

    #[test]
    fn inference_prompt_contains_trace_context_and_tool_contract() {
        let trace = vec![TraceEvent {
            phase: "prompt".to_string(),
            content: "fix tests".to_string(),
        }];
        let prompt = build_inference_prompt(
            &trace,
            "fn main()",
            "fix tests",
            Some("# Skill\nUse tests"),
            Some("- prior session clue"),
            Some("Goal: fix tests"),
        );

        assert!(prompt.contains("fn main()"));
        assert!(prompt.contains("\"phase\":\"prompt\""));
        assert!(prompt.contains("\"tool\":\"run_command\""));
        assert!(prompt.contains("search_viking_context"));
        assert!(prompt.contains("PAST EXPERIENCE"));
        assert!(prompt.contains("RECENT SWARM INTELLIGENCE"));
        assert!(prompt.contains("[APPROVED PLAN]"));
    }

    #[test]
    fn parses_json_tool_call_inside_model_text() {
        let call = parse_tool_call(
            "next:\n{\"tool\":\"edit_file\",\"path\":\"src/lib.rs\",\"contents\":\"ok\"}",
        )
        .unwrap();

        assert_eq!(
            call,
            ToolCall::EditFile {
                path: "src/lib.rs".to_string(),
                contents: "ok".to_string()
            }
        );
    }

    #[test]
    fn parses_new_tool_calls() {
        assert!(matches!(
            parse_tool_call(r#"{"tool":"search_viking_context","query":"SessionConfig"}"#).unwrap(),
            ToolCall::SearchVikingContext { .. }
        ));
        assert!(matches!(
            parse_tool_call(
                r#"{"tool":"request_human_action","instruction":"check UI","requires_text_feedback":true}"#
            )
            .unwrap(),
            ToolCall::RequestHumanAction { .. }
        ));
    }

    #[test]
    fn session_records_trace_as_json() {
        let mut session = Session::new(config()).unwrap();
        session.record_user_input("fix faas/orchestrator/src/lib.rs");
        session.record_observation("run_command", "ok");

        let json = session.trace_json().unwrap();
        assert!(json.contains("fix faas/orchestrator/src/lib.rs"));
        assert!(json.contains("run_command"));
    }

    #[test]
    fn prompt_path_hint_becomes_viking_uri() {
        assert_eq!(
            viking_uri_for_prompt("inspect `faas/orchestrator/src/lib.rs`, please"),
            "viking://faas/orchestrator/src/lib.rs"
        );
    }

    #[test]
    fn tier_selects_expected_model() {
        assert_eq!(model_for_tier(1), "qwen-coder-7b");
        assert_eq!(model_for_tier(2), "qwen-coder-27b");
    }

    #[test]
    fn skill_matching_uses_keyword_overlap() {
        let skills = vec![
            "refactor-parser.md".to_string(),
            "deploy-cloud.md".to_string(),
        ];

        assert_eq!(
            match_skill("please refactor the parser", &skills),
            Some("refactor-parser.md".to_string())
        );
    }

    #[test]
    fn workspace_index_filter_excludes_heavy_and_hidden_paths() {
        assert!(is_indexable_workspace_path("src/lib.rs"));
        assert!(!is_indexable_workspace_path("target/debug/app"));
        assert!(!is_indexable_workspace_path(".git/config"));
        assert!(!is_indexable_workspace_path("node_modules/pkg/index.js"));
    }

    #[test]
    fn command_failures_escalate_on_same_target() {
        let mut session = Session::new(config()).unwrap();
        for _ in 0..3 {
            session.record_command_result("cargo test", "error: failed");
        }

        assert_eq!(session.consecutive_failures, FAILURE_ESCALATION_THRESHOLD);
        session.record_command_result("cargo test", "test result: ok");
        assert_eq!(session.consecutive_failures, 0);
    }

    #[test]
    fn mcp_keys_sort_by_timestamp() {
        assert!(sortable_timestamp_key(1) < sortable_timestamp_key(2));
        assert_eq!(
            mcp_observation_key("sess", 1),
            "v1:mcp:0000000000000001:sess"
        );
    }

    #[test]
    fn plan_can_be_pinned_into_prompt() {
        let plan = Plan {
            goal: "fix tests".to_string(),
            branch: Some("codex/fix-tests".to_string()),
            files: vec!["src/lib.rs".to_string()],
            steps: vec!["run cargo test".to_string()],
        };

        let pinned = plan.pinned_text();
        assert!(pinned.contains("fix tests"));
        assert!(pinned.contains("src/lib.rs"));
    }
}
