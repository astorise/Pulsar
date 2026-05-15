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
    use tachyon::ai::{
        human_bridge, inference, kv_partition, skill_extractor, viking_context, workspace_bridge,
    };

    static SESSIONS: LazyLock<Mutex<Vec<super::Session>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));

    struct Orchestrator;

    impl Guest for Orchestrator {
        fn start_session(cfg: SessionConfig) -> Result<String, String> {
            super::init_wasm_tracing();
            let _workspace_token = super::resolve_workspace_token(Some(&cfg.workspace_token))?;
            let config = super::SessionConfig {
                workspace_url: cfg.workspace_url,
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
            session.active_skill = past_skill.as_ref().map(|skill| skill.adapter_id.clone());

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
                past_skill.as_ref().map(|skill| skill.content.as_str()),
                non_empty(&swarm_intel),
                approved_plan.as_deref(),
            );
            let response = generate_with_escalation(session, prompt, 1024)?;
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
                let guard = super::PathGuard::workspace();
                let guarded_uri =
                    super::guarded_viking_uri(&uri, &guard).map_err(super::format_tool_error)?;
                let resolved = viking_context::resolve(
                    &guarded_uri,
                    viking_context::ContextLevel::L1Structure,
                )?;
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
                let guard = super::PathGuard::workspace();
                let guarded_path = guard.validate(&path).map_err(super::format_tool_error)?;
                let hover =
                    workspace_bridge::lsp_hover(&session.id, &guarded_path, line, character)?;
                session.record_observation("ask_lsp", &hover);
            }
            super::ToolCall::EditFile { path, contents } => {
                if !session.has_approved_plan() {
                    session
                        .record_observation("planning_required", "You must use submit_plan first");
                    return Ok(());
                }
                let guard = super::PathGuard::workspace();
                let guarded_path = guard.validate(&path).map_err(super::format_tool_error)?;
                let impact = super::impact_for_edit_file(&guarded_path, &contents);
                let Some(contents) =
                    request_human_approval_for_edit(session, &guarded_path, contents, impact)?
                else {
                    return Ok(());
                };
                workspace_bridge::webdav_put(&session.id, &guarded_path, contents.as_bytes())?;
                session.record_observation("edit_file", &format!("updated {guarded_path}"));
                let _ = broadcast_to_swarm(session, &format!("updated {guarded_path}"));
            }
            super::ToolCall::RunCommand {
                command,
                executable,
                args,
            } => {
                if !session.has_approved_plan() {
                    session
                        .record_observation("planning_required", "You must use submit_plan first");
                    return Ok(());
                }
                let command_request = super::CommandRequest::from_tool(command, executable, args)
                    .map_err(super::format_tool_error)?;
                command_request
                    .validate()
                    .map_err(super::format_tool_error)?;
                let impact = super::impact_for_command_request(&command_request);
                let Some(command_request) =
                    request_human_approval_for_command(session, command_request, impact)?
                else {
                    return Ok(());
                };
                let command = command_request.display();
                let command_result = workspace_bridge::websocket_command(
                    &session.id,
                    command_request.executable(),
                    command_request.args(),
                );
                let (exit_code, raw) = match command_result {
                    Ok(result) => (
                        result.exit_code,
                        super::format_command_result(
                            result.exit_code,
                            &result.stdout,
                            &result.stderr,
                        ),
                    ),
                    Err(err) => (-1, format!("ERROR: {err}")),
                };
                let cleaned = super::clean_output(&command, &raw);
                session.record_command_result_status(&command, exit_code, &cleaned);
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

    fn request_human_approval_for_edit(
        session: &mut super::Session,
        path: &str,
        contents: String,
        impact: u8,
    ) -> Result<Option<String>, String> {
        if impact <= super::HUMAN_APPROVAL_THRESHOLD {
            return Ok(Some(contents));
        }

        let summary = super::human_approval_summary("edit_file", path, impact);
        match human_bridge::request_approval(&session.id, &summary, impact)? {
            human_bridge::ApprovalStatus::Approved => Ok(Some(contents)),
            human_bridge::ApprovalStatus::Rejected(reason) => {
                session.status = super::AgentStatus::AwaitingUser;
                session.record_observation("operation_rejected_by_human", &reason);
                Ok(None)
            }
            human_bridge::ApprovalStatus::Modified(modified) => {
                session.record_observation(
                    "operation_modified_by_human",
                    &format!("edit_file {path}"),
                );
                Ok(Some(modified))
            }
        }
    }

    fn request_human_approval_for_command(
        session: &mut super::Session,
        command_request: super::CommandRequest,
        impact: u8,
    ) -> Result<Option<super::CommandRequest>, String> {
        if impact <= super::HUMAN_APPROVAL_THRESHOLD {
            return Ok(Some(command_request));
        }

        let summary =
            super::human_approval_summary("run_command", &command_request.display(), impact);
        match human_bridge::request_approval(&session.id, &summary, impact)? {
            human_bridge::ApprovalStatus::Approved => Ok(Some(command_request)),
            human_bridge::ApprovalStatus::Rejected(reason) => {
                session.status = super::AgentStatus::AwaitingUser;
                session.record_observation("operation_rejected_by_human", &reason);
                Ok(None)
            }
            human_bridge::ApprovalStatus::Modified(modified) => {
                let replacement = super::apply_modified_command(command_request, &modified)
                    .map_err(super::format_tool_error)?;
                session.record_observation("operation_modified_by_human", &replacement.display());
                Ok(Some(replacement))
            }
        }
    }

    fn crawl_workspace(session_id: &str) -> Result<(), String> {
        let guard = super::PathGuard::workspace();
        let root = guard.validate(".").map_err(super::format_tool_error)?;
        let entries = workspace_bridge::webdav_propfind(session_id, &root)?;
        for entry in entries
            .iter()
            .filter_map(|entry| guard.validate(entry).ok())
            .filter(|entry| super::is_indexable_workspace_path(entry))
        {
            let uri = format!("viking://{entry}");
            let _ = viking_context::resolve(&uri, viking_context::ContextLevel::L0Summary);
        }
        Ok(())
    }

    fn discover_skill(session_id: &str, prompt: &str) -> Option<super::SkillSelection> {
        let guard = super::PathGuard::workspace();
        let skills_path = guard.validate(".pulsar/skills").ok()?;
        let skills = workspace_bridge::webdav_propfind(session_id, &skills_path).ok()?;
        let selected = super::match_skill(prompt, &skills)?;
        let path = if selected.starts_with(".pulsar/skills/") {
            selected.clone()
        } else {
            format!(".pulsar/skills/{selected}")
        };
        let guarded_path = guard.validate(&path).ok()?;
        let bytes = workspace_bridge::webdav_get(session_id, &guarded_path).ok()?;
        Some(super::SkillSelection {
            adapter_id: super::adapter_id_from_skill_path(&selected),
            content: String::from_utf8(bytes).ok()?,
        })
    }

    fn generate_with_escalation(
        session: &mut super::Session,
        prompt: String,
        max_tokens: u32,
    ) -> Result<inference::InferenceResponse, String> {
        let tiers = super::tiers_for_max(session.config.max_tier);
        let mut last_error = None;

        for (idx, tier) in tiers.iter().enumerate() {
            let response = inference::generate(&inference::InferenceRequest {
                model_id: tier.model.to_string(),
                prompt: prompt.clone(),
                max_tokens,
                temperature: 0.1,
                lora_adapter: session.active_skill.clone(),
            });

            match response {
                Ok(response) if super::response_needs_escalation(&response.text) => {
                    if let Some(next) = tiers.get(idx + 1) {
                        session.record_escalation(tier.model, next.model, "rabbit_hole_detected");
                        tracing::info!(
                            from_tier = tier.model,
                            to_tier = next.model,
                            reason = "rabbit_hole_detected",
                            "orchestrator inference escalation"
                        );
                        continue;
                    }
                    return Ok(response);
                }
                Ok(response) => return Ok(response),
                Err(err) if super::error_needs_escalation(&err) => {
                    if let Some(next) = tiers.get(idx + 1) {
                        session.record_escalation(tier.model, next.model, &err);
                        tracing::info!(
                            from_tier = tier.model,
                            to_tier = next.model,
                            reason = err.as_str(),
                            "orchestrator inference escalation"
                        );
                        last_error = Some(err);
                        continue;
                    }
                    return Err(err);
                }
                Err(err) => return Err(err),
            }
        }

        Err(last_error.unwrap_or_else(|| "inference escalation exhausted".to_string()))
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
use std::path::{Component, Path, PathBuf};
use std::sync::Once;

pub const FAILURE_ESCALATION_THRESHOLD: u32 = 3;
pub const HUMAN_APPROVAL_THRESHOLD: u8 = 5;

static TELEMETRY_INIT: Once = Once::new();

pub fn init_wasm_tracing() {
    TELEMETRY_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .json()
            .with_target(true)
            .with_current_span(false)
            .without_time()
            .try_init();
        tracing::info!(
            target = "tachyon.mesh.telemetry",
            event = "orchestrator_telemetry_initialized"
        );
    });
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConfig {
    pub workspace_url: String,
    pub max_tier: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSelection {
    pub adapter_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelTier {
    pub level: u32,
    pub model: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    UnauthorizedAccess { path: String },
    CommandForbidden { command: String },
    CommandInjection { command: String },
    InvalidCommand { message: String },
}

impl ToolError {
    pub fn reason(&self) -> &'static str {
        match self {
            Self::UnauthorizedAccess { .. } => "UnauthorizedAccess",
            Self::CommandForbidden { .. } => "CommandForbidden",
            Self::CommandInjection { .. } => "CommandInjectionDetected",
            Self::InvalidCommand { .. } => "InvalidCommand",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::UnauthorizedAccess { path } => {
                format!("Access to '{path}' is forbidden. You are restricted to the workspace directory.")
            }
            Self::CommandForbidden { command } => {
                format!("Command '{command}' is not in the orchestrator allowlist.")
            }
            Self::CommandInjection { command } => {
                format!("Command '{command}' contains shell control operators and was rejected.")
            }
            Self::InvalidCommand { message } => message.clone(),
        }
    }
}

pub fn format_tool_error(error: ToolError) -> String {
    serde_json::json!({
        "status": "error",
        "reason": error.reason(),
        "message": error.message(),
    })
    .to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathGuard {
    workspace_root: PathBuf,
}

impl PathGuard {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    pub fn workspace() -> Self {
        Self::new(".")
    }

    pub fn validate(&self, raw_path: &str) -> Result<String, ToolError> {
        if raw_path.contains('\0') || raw_path.contains('\\') {
            return Err(ToolError::UnauthorizedAccess {
                path: raw_path.to_string(),
            });
        }

        let path = Path::new(raw_path);
        if path.is_absolute() {
            return Err(ToolError::UnauthorizedAccess {
                path: raw_path.to_string(),
            });
        }

        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::Normal(part) => normalized.push(part),
                Component::CurDir => {}
                Component::ParentDir => {
                    if !normalized.pop() {
                        return Err(ToolError::UnauthorizedAccess {
                            path: raw_path.to_string(),
                        });
                    }
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(ToolError::UnauthorizedAccess {
                        path: raw_path.to_string(),
                    });
                }
            }
        }

        if normalized.as_os_str().is_empty() {
            return Ok(".".to_string());
        }

        let joined = self.workspace_root.join(&normalized);
        if joined
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(ToolError::UnauthorizedAccess {
                path: raw_path.to_string(),
            });
        }

        Ok(normalized.to_string_lossy().replace('\\', "/"))
    }
}

pub fn guarded_viking_uri(raw_uri: &str, guard: &PathGuard) -> Result<String, ToolError> {
    let path = raw_uri
        .strip_prefix("viking://")
        .ok_or_else(|| ToolError::UnauthorizedAccess {
            path: raw_uri.to_string(),
        })?;
    guard.validate(path).map(|path| format!("viking://{path}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRequest {
    executable: String,
    args: Vec<String>,
}

impl CommandRequest {
    pub fn from_tool(
        command: Option<String>,
        executable: Option<String>,
        args: Vec<String>,
    ) -> Result<Self, ToolError> {
        if let Some(executable) = executable {
            return Ok(Self { executable, args });
        }

        let command = command.ok_or_else(|| ToolError::InvalidCommand {
            message: "run_command requires either `command` or `cmd`.".to_string(),
        })?;
        parse_legacy_command(&command)
    }

    pub fn validate(&self) -> Result<(), ToolError> {
        if self.executable.trim().is_empty() {
            return Err(ToolError::InvalidCommand {
                message: "command executable must not be empty".to_string(),
            });
        }

        if contains_shell_operator(&self.executable) || self.executable.contains('\0') {
            return Err(ToolError::CommandInjection {
                command: self.display(),
            });
        }

        if !is_allowed_command(&self.executable, &self.args) {
            return Err(ToolError::CommandForbidden {
                command: self.executable.clone(),
            });
        }

        if self
            .args
            .iter()
            .any(|arg| contains_shell_operator(arg) || arg.contains('\0'))
        {
            return Err(ToolError::CommandInjection {
                command: self.display(),
            });
        }

        Ok(())
    }

    pub fn executable(&self) -> &str {
        &self.executable
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn display(&self) -> String {
        std::iter::once(self.executable.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
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
        #[serde(default)]
        command: Option<String>,
        #[serde(default, alias = "cmd")]
        executable: Option<String>,
        #[serde(default)]
        args: Vec<String>,
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
        self.record_command_result_status(
            command,
            command_exit_code_for_legacy_output(output),
            output,
        );
    }

    pub fn record_command_result_status(&mut self, command: &str, exit_code: i32, _output: &str) {
        let target = command_failure_target(command);
        if command_exit_code_succeeded(exit_code) {
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

    pub fn record_escalation(&mut self, from_tier: &str, to_tier: &str, reason: &str) {
        self.record_observation(
            "inference_escalation",
            &format!("{from_tier} -> {to_tier}: {reason}"),
        );
    }

    pub fn trace_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.trace).map_err(|err| err.to_string())
    }
}

pub fn validate_session_config(config: &SessionConfig) -> Result<(), String> {
    if normalize_workspace_url(&config.workspace_url)?.is_empty() {
        return Err("workspace-url must not be empty".to_string());
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
        .chain(config.max_tier.to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("sess-{hash:016x}")
}

pub fn resolve_workspace_token(provided: Option<&str>) -> Result<String, String> {
    match provided.map(str::trim).filter(|token| !token.is_empty()) {
        Some(token) => Ok(token.to_string()),
        None => workspace_token_from_env(),
    }
}

pub fn workspace_token_from_env() -> Result<String, String> {
    std::env::var("WORKSPACE_TOKEN")
        .map(|token| token.trim().to_string())
        .map_err(|_| "workspace-token must be provided by host or WORKSPACE_TOKEN".to_string())
        .and_then(|token| {
            if token.is_empty() {
                Err("workspace-token must not be empty".to_string())
            } else {
                Ok(token)
            }
        })
}

pub fn authorization_header_from_env() -> Result<String, String> {
    resolve_workspace_token(None).map(|token| format!("Bearer {token}"))
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
{{"tool":"run_command","cmd":"cargo","args":["test"]}}
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
    match max_tier {
        0 | 1 => "qwen-coder-7b",
        2 => "qwen-coder-27b",
        _ => "claude-3-opus",
    }
}

pub fn tiers_for_max(max_tier: u32) -> Vec<ModelTier> {
    (1..=max_tier.clamp(1, 3))
        .map(|level| ModelTier {
            level,
            model: model_for_tier(level),
        })
        .collect()
}

pub fn response_needs_escalation(text: &str) -> bool {
    text.contains("<rabbit_hole_detected>")
}

pub fn error_needs_escalation(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("resourceexhausted")
        || normalized.contains("resource exhausted")
        || normalized.contains("timeout")
        || normalized.contains("timed out")
}

pub fn adapter_id_from_skill_path(path: &str) -> String {
    let file_name = match path.rsplit('/').next() {
        Some(name) => name,
        None => path,
    };
    file_name
        .trim_end_matches(".md")
        .trim_end_matches(".json")
        .to_string()
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
    command_exit_code_for_legacy_output(output) == 0
}

fn command_exit_code_for_legacy_output(output: &str) -> i32 {
    let lower = output.to_ascii_lowercase();
    if lower.contains("error:")
        || lower.contains("failed")
        || lower.contains("panicked")
        || lower.contains("exit code")
        || lower.contains("traceback")
    {
        1
    } else {
        0
    }
}

pub fn command_exit_code_succeeded(exit_code: i32) -> bool {
    exit_code == 0
}

pub fn format_command_result(exit_code: i32, stdout: &str, stderr: &str) -> String {
    match (stdout.is_empty(), stderr.is_empty()) {
        (false, true) => stdout.to_string(),
        (true, false) => format!("exit code {exit_code}\n{stderr}"),
        (false, false) => format!("exit code {exit_code}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"),
        (true, true) => format!("exit code {exit_code}"),
    }
}

pub fn command_failure_target(command: &str) -> String {
    command
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_legacy_command(command: &str) -> Result<CommandRequest, ToolError> {
    if contains_shell_operator(command) || command.contains('\0') {
        return Err(ToolError::CommandInjection {
            command: command.to_string(),
        });
    }

    let parts = shell_words::split(command).map_err(|err| ToolError::InvalidCommand {
        message: format!("failed to parse command: {err}"),
    })?;
    let mut parts = parts.into_iter();
    let executable = parts.next().ok_or_else(|| ToolError::InvalidCommand {
        message: "command must not be empty".to_string(),
    })?;
    if executable.trim().is_empty() {
        return Err(ToolError::InvalidCommand {
            message: "command executable must not be empty".to_string(),
        });
    }

    let args = parts.collect::<Vec<_>>();
    if contains_shell_operator(&executable)
        || executable.contains('\0')
        || args
            .iter()
            .any(|arg| contains_shell_operator(arg) || arg.contains('\0'))
    {
        return Err(ToolError::CommandInjection {
            command: command.to_string(),
        });
    }

    Ok(CommandRequest { executable, args })
}

pub fn impact_for_tool_call(tool_call: &ToolCall) -> u8 {
    match tool_call {
        ToolCall::EditFile { path, contents } => impact_for_edit_file(path, contents),
        ToolCall::RunCommand {
            command,
            executable,
            args,
        } => CommandRequest::from_tool(command.clone(), executable.clone(), args.clone())
            .map(|request| impact_for_command_request(&request))
            .unwrap_or(10),
        ToolCall::ReadVikingContext { .. }
        | ToolCall::SearchVikingContext { .. }
        | ToolCall::QueryGraph { .. }
        | ToolCall::AskLsp { .. } => 0,
        ToolCall::SubmitPlan { .. } | ToolCall::RequestHumanAction { .. } => 1,
        ToolCall::Finish { .. } => 0,
    }
}

pub fn impact_for_edit_file(path: &str, contents: &str) -> u8 {
    let path = path.replace('\\', "/").to_ascii_lowercase();
    let mut impact = 6;

    if path.starts_with(".github/")
        || path == "cargo.toml"
        || path == "cargo.lock"
        || path == "rust-toolchain.toml"
        || path.ends_with("/cargo.toml")
    {
        impact = impact.max(8);
    }

    if path.ends_with(".ps1")
        || path.ends_with(".sh")
        || contents.contains("Remove-Item")
        || contents.contains("rm -rf")
    {
        impact = impact.max(9);
    }

    impact
}

pub fn impact_for_command_request(command: &CommandRequest) -> u8 {
    let executable = command.executable();
    let args = command.args();
    let first_arg = args.first().map(String::as_str).unwrap_or_default();

    match executable {
        "git" => match first_arg {
            "status" | "diff" | "log" | "show" | "branch" => 1,
            "add" => 6,
            "commit" | "push" | "pull" | "fetch" | "rebase" | "merge" | "tag" | "checkout"
            | "switch" => 8,
            _ => 7,
        },
        "cargo" => match first_arg {
            "test" | "check" | "clippy" | "fmt" | "build" => 3,
            "publish" | "install" => 8,
            _ => 5,
        },
        "npm" => match first_arg {
            "test" | "run" | "build" => 4,
            "install" | "ci" => 6,
            "publish" => 10,
            _ => 5,
        },
        "node" | "rustc" => 3,
        "ls" | "cat" | "rg" | "echo" => 1,
        _ => 10,
    }
}

pub fn human_approval_summary(operation: &str, target: &str, impact: u8) -> String {
    format!("Human approval required for {operation} on `{target}` (impact {impact}/10)")
}

pub fn apply_modified_command(
    original: CommandRequest,
    modified: &str,
) -> Result<CommandRequest, ToolError> {
    if modified.trim().is_empty() {
        return Ok(original);
    }

    let replacement = parse_legacy_command(modified)?;
    replacement.validate()?;
    Ok(replacement)
}

pub fn is_allowed_command(command: &str, args: &[String]) -> bool {
    if command == "git" {
        return is_allowed_git_command(args);
    }

    matches!(
        command,
        "cargo" | "npm" | "node" | "rustc" | "ls" | "cat" | "rg" | "echo"
    )
}

pub fn is_allowed_git_command(args: &[String]) -> bool {
    if args.iter().any(|arg| {
        arg == "-C"
            || arg == "--exec-path"
            || arg.starts_with("-c")
            || arg.starts_with("--exec-path=")
    }) {
        return false;
    }

    let Some(verb) = args.first().map(String::as_str) else {
        return false;
    };

    matches!(
        verb,
        "status"
            | "diff"
            | "log"
            | "add"
            | "commit"
            | "branch"
            | "switch"
            | "push"
            | "pull"
            | "fetch"
            | "rebase"
            | "merge"
            | "tag"
            | "show"
            | "checkout"
    )
}

pub fn contains_shell_operator(value: &str) -> bool {
    [";", "|", "&&", "||", "`", "$(", ">", "<"]
        .iter()
        .any(|operator| value.contains(operator))
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

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("unexpected error: {err:?}"),
        }
    }

    fn config() -> SessionConfig {
        SessionConfig {
            workspace_url: "http://127.0.0.1:49152/workspace/".to_string(),
            max_tier: 2,
        }
    }

    #[test]
    fn session_config_validates_url_and_tier_without_persisted_token() {
        assert!(validate_session_config(&config()).is_ok());

        let mut invalid = config();
        invalid.max_tier = 0;
        match validate_session_config(&invalid) {
            Ok(()) => panic!("expected max-tier validation failure"),
            Err(err) => assert_eq!(err, "max-tier must be at least 1"),
        }

        let session = must(Session::new(config()));
        assert_eq!(
            session.config.workspace_url,
            "http://127.0.0.1:49152/workspace/"
        );
    }

    #[test]
    fn workspace_token_is_resolved_then_dropped_from_session_config() {
        let resolved = must(resolve_workspace_token(Some(" secret ")));
        assert_eq!(resolved, "secret");

        let session = must(Session::new(config()));
        let serialized = must(serde_json::to_string(&session));
        assert!(!serialized.contains("secret"));
        assert!(!serialized.contains("workspace_token"));
    }

    #[test]
    fn webdav_paths_are_joined_without_double_slash() {
        assert_eq!(
            must(webdav_path("http://localhost:8080/root/", "/src/main.rs")),
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
        );
        let call = must(call);

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
            must(parse_tool_call(
                r#"{"tool":"search_viking_context","query":"SessionConfig"}"#
            )),
            ToolCall::SearchVikingContext { .. }
        ));
        assert!(matches!(
            must(parse_tool_call(
                r#"{"tool":"request_human_action","instruction":"check UI","requires_text_feedback":true}"#
            )),
            ToolCall::RequestHumanAction { .. }
        ));
        assert!(matches!(
            must(parse_tool_call(
                r#"{"tool":"run_command","cmd":"cargo","args":["test"]}"#
            )),
            ToolCall::RunCommand { .. }
        ));
    }

    #[test]
    fn session_records_trace_as_json() {
        let mut session = must(Session::new(config()));
        session.record_user_input("fix faas/orchestrator/src/lib.rs");
        session.record_observation("run_command", "ok");

        let json = must(session.trace_json());
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
        assert_eq!(model_for_tier(3), "claude-3-opus");

        let tiers = tiers_for_max(3);
        assert_eq!(tiers.len(), 3);
        assert_eq!(tiers[0].model, "qwen-coder-7b");
        assert_eq!(tiers[2].model, "claude-3-opus");
    }

    #[test]
    fn escalation_triggers_are_detected() {
        assert!(response_needs_escalation(
            "I am stuck <rabbit_hole_detected>"
        ));
        assert!(error_needs_escalation("ResourceExhausted: no vram"));
        assert!(error_needs_escalation("request timed out"));
        assert!(!error_needs_escalation("invalid prompt"));
    }

    #[test]
    fn skill_path_becomes_adapter_id() {
        assert_eq!(
            adapter_id_from_skill_path(".pulsar/skills/refactor-parser.md"),
            "refactor-parser"
        );
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
        let mut session = must(Session::new(config()));
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
    fn path_guard_blocks_traversal_and_absolute_paths() {
        let guard = PathGuard::workspace();

        assert!(matches!(
            guard.validate("../../../etc/passwd"),
            Err(ToolError::UnauthorizedAccess { .. })
        ));
        assert!(matches!(
            guard.validate("..\\etc\\passwd"),
            Err(ToolError::UnauthorizedAccess { .. })
        ));
        assert!(matches!(
            guard.validate("/etc/passwd"),
            Err(ToolError::UnauthorizedAccess { .. })
        ));
        assert!(matches!(
            guard.validate("src/lib.rs\0"),
            Err(ToolError::UnauthorizedAccess { .. })
        ));
        assert_eq!(
            must(guard.validate("./faas/orchestrator/src/lib.rs")),
            "faas/orchestrator/src/lib.rs"
        );
        assert_eq!(must(guard.validate("src/../Cargo.toml")), "Cargo.toml");
        assert_eq!(
            must(guarded_viking_uri("viking://src/../Cargo.toml", &guard)),
            "viking://Cargo.toml"
        );
        assert!(matches!(
            guarded_viking_uri("file://Cargo.toml", &guard),
            Err(ToolError::UnauthorizedAccess { .. })
        ));
    }

    #[test]
    fn command_sandbox_blocks_injection_and_disallowed_commands() {
        assert!(matches!(
            parse_legacy_command(r#"echo "test"; rm -rf /"#),
            Err(ToolError::CommandInjection { .. })
        ));
        assert!(matches!(
            parse_legacy_command(r#""""#),
            Err(ToolError::InvalidCommand { .. })
        ));
        assert!(matches!(
            parse_legacy_command("\u{1c}\n'&'&"),
            Err(ToolError::CommandInjection { .. })
        ));

        let forbidden = CommandRequest::from_tool(
            None,
            Some("powershell".to_string()),
            vec!["Remove-Item".to_string()],
        );
        let forbidden = must(forbidden);
        assert!(matches!(
            forbidden.validate(),
            Err(ToolError::CommandForbidden { .. })
        ));

        let allowed = CommandRequest::from_tool(
            None,
            Some("cargo".to_string()),
            vec!["test".to_string(), "--workspace".to_string()],
        );
        let allowed = must(allowed);
        must(allowed.validate());
        assert_eq!(allowed.display(), "cargo test --workspace");
    }

    #[test]
    fn command_sandbox_restricts_git_configuration_and_verbs() {
        let allowed = must(CommandRequest::from_tool(
            None,
            Some("git".to_string()),
            vec!["status".to_string(), "--short".to_string()],
        ));
        assert!(allowed.validate().is_ok());

        let blocked_config = must(CommandRequest::from_tool(
            None,
            Some("git".to_string()),
            vec![
                "-c".to_string(),
                "core.sshCommand=bad".to_string(),
                "status".to_string(),
            ],
        ));
        assert!(matches!(
            blocked_config.validate(),
            Err(ToolError::CommandForbidden { .. })
        ));

        let blocked_verb = must(CommandRequest::from_tool(
            None,
            Some("git".to_string()),
            vec![
                "config".to_string(),
                "--global".to_string(),
                "alias.x".to_string(),
            ],
        ));
        assert!(matches!(
            blocked_verb.validate(),
            Err(ToolError::CommandForbidden { .. })
        ));
    }

    #[test]
    fn impact_scorer_flags_state_mutating_operations() {
        let read = ToolCall::ReadVikingContext {
            uri: "viking://src/lib.rs".to_string(),
        };
        assert_eq!(impact_for_tool_call(&read), 0);

        let edit = ToolCall::EditFile {
            path: "src/lib.rs".to_string(),
            contents: "fn main() {}".to_string(),
        };
        assert!(impact_for_tool_call(&edit) > HUMAN_APPROVAL_THRESHOLD);

        let status = must(CommandRequest::from_tool(
            None,
            Some("git".to_string()),
            vec!["status".to_string(), "--short".to_string()],
        ));
        assert!(impact_for_command_request(&status) <= HUMAN_APPROVAL_THRESHOLD);

        let push = must(CommandRequest::from_tool(
            None,
            Some("git".to_string()),
            vec!["push".to_string()],
        ));
        assert!(impact_for_command_request(&push) > HUMAN_APPROVAL_THRESHOLD);
    }

    #[test]
    fn modified_human_command_is_reparsed_and_revalidated() {
        let original = must(CommandRequest::from_tool(
            None,
            Some("git".to_string()),
            vec!["push".to_string()],
        ));
        let modified = must(apply_modified_command(original, "cargo test --workspace"));

        assert_eq!(modified.executable(), "cargo");
        assert_eq!(
            modified.args(),
            &["test".to_string(), "--workspace".to_string()]
        );
        assert!(modified.validate().is_ok());
    }

    #[test]
    fn command_success_uses_exit_code() {
        assert!(command_exit_code_succeeded(0));
        assert!(!command_exit_code_succeeded(1));

        let mut session = must(Session::new(config()));
        session.record_command_result_status("cargo test", 1, "all text looks fine");
        assert_eq!(session.consecutive_failures, 1);

        session.record_command_result_status("cargo test", 0, "error: stale text");
        assert_eq!(session.consecutive_failures, 0);
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
