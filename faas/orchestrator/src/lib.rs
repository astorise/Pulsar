#[cfg(target_arch = "wasm32")]
mod component {
    wit_bindgen::generate!({
        path: "wit",
        world: "orchestrator-world",
    });

    use exports::tachyon::ai::orchestrator::{Guest, SessionConfig};
    use std::sync::{LazyLock, Mutex};
    use tachyon::ai::{inference, skill_extractor, viking_context, workspace_bridge};

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

            let prompt = super::build_inference_prompt(&session.trace, &context.payload, &text);
            let response = inference::generate(&inference::InferenceRequest {
                model_id: super::model_for_tier(session.config.max_tier).to_string(),
                prompt,
                max_tokens: 1024,
                temperature: 0.1,
                lora_adapter: None,
            })?;

            session.status = super::AgentStatus::Acting;
            let tool_call = super::parse_tool_call(&response.text)?;
            match tool_call {
                super::ToolCall::ReadVikingContext { uri } => {
                    let resolved =
                        viking_context::resolve(&uri, viking_context::ContextLevel::L1Structure)?;
                    session.record_observation("read_viking_context", &resolved.payload);
                }
                super::ToolCall::EditFile { path, contents } => {
                    workspace_bridge::webdav_put(&session.id, &path, contents.as_bytes())?;
                    session.record_observation("edit_file", &format!("updated {path}"));
                }
                super::ToolCall::RunCommand { command } => {
                    let output = workspace_bridge::websocket_command(&session.id, &command)?;
                    session.record_observation("run_command", &output);
                }
                super::ToolCall::Finish { summary } => {
                    session.status = super::AgentStatus::Finished;
                    session.record_observation("finish", &summary);
                    let trace_json = session.trace_json()?;
                    let _ = skill_extractor::extract(&skill_extractor::ExtractionRequest {
                        task_description: text,
                        execution_trace: trace_json,
                    });
                }
            }

            if !matches!(session.status, super::AgentStatus::Finished) {
                session.status = super::AgentStatus::AwaitingUser;
            }

            Ok(())
        }
    }

    export!(Orchestrator);
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfig {
    pub workspace_url: String,
    pub workspace_token: String,
    pub max_tier: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Acting,
    AwaitingUser,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub phase: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub config: SessionConfig,
    pub status: AgentStatus,
    pub trace: Vec<TraceEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum ToolCall {
    ReadVikingContext { uri: String },
    EditFile { path: String, contents: String },
    RunCommand { command: String },
    Finish { summary: String },
}

impl Session {
    pub fn new(config: SessionConfig) -> Result<Self, String> {
        validate_session_config(&config)?;
        Ok(Self {
            id: build_session_id(&config),
            config,
            status: AgentStatus::Idle,
            trace: Vec::new(),
        })
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

pub fn build_inference_prompt(trace: &[TraceEvent], context: &str, user_input: &str) -> String {
    let trace_json = serde_json::to_string(trace).unwrap_or_else(|_| "[]".to_string());
    format!(
        r#"You are the Pulsar orchestrator. Decide the next tool call.

User input:
{user_input}

Semantic context:
{context}

Execution trace:
{trace_json}

Return exactly one JSON object using one of these forms:
{{"tool":"read_viking_context","uri":"viking://path"}}
{{"tool":"edit_file","path":"repo/path","contents":"new file contents"}}
{{"tool":"run_command","command":"cargo test"}}
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
        let prompt = build_inference_prompt(&trace, "fn main()", "fix tests");

        assert!(prompt.contains("fn main()"));
        assert!(prompt.contains("\"phase\":\"prompt\""));
        assert!(prompt.contains("\"tool\":\"run_command\""));
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
}
