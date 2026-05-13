use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Init {
        workspace_url: String,
        workspace_token: String,
    },
    UserMessage {
        content: String,
    },
    LspHoverResponse {
        id: String,
        markdown: String,
    },
    ResumeRequest {
        session_id: String,
        feedback: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    StreamToken {
        content: String,
    },
    ActionEvent {
        action: String,
        target: String,
    },
    LspHoverRequest {
        id: String,
        path: String,
        line: u32,
        character: u32,
    },
    Suspend {
        instruction: String,
        requires_feedback: bool,
    },
    Handshake {
        plan: serde_json::Value,
    },
    Escalated {
        report: String,
    },
    Kiln {
        message: String,
        dataset_size: u32,
        training_submitted: bool,
    },
    Error {
        message: String,
    },
}

pub fn encode_client_message(message: &ClientMessage) -> Result<String, serde_json::Error> {
    serde_json::to_string(message)
}

pub fn decode_server_message(payload: &str) -> Result<ServerMessage, serde_json::Error> {
    serde_json::from_str(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_message_uses_expected_json_shape() {
        let json = encode_client_message(&ClientMessage::Init {
            workspace_url: "http://127.0.0.1:9000/webdav".to_string(),
            workspace_token: "secret".to_string(),
        })
        .unwrap();

        assert!(json.contains("\"type\":\"init\""));
        assert!(json.contains("\"workspace_url\":\"http://127.0.0.1:9000/webdav\""));
        assert!(json.contains("\"workspace_token\":\"secret\""));
    }

    #[test]
    fn server_stream_token_decodes() {
        let msg = decode_server_message(r#"{"type":"stream_token","content":"hello"}"#).unwrap();

        assert_eq!(
            msg,
            ServerMessage::StreamToken {
                content: "hello".to_string()
            }
        );
    }

    #[test]
    fn lsp_hover_messages_roundtrip() {
        let response = encode_client_message(&ClientMessage::LspHoverResponse {
            id: "hover-1".to_string(),
            markdown: "fn run()".to_string(),
        })
        .unwrap();
        assert!(response.contains("\"type\":\"lsp_hover_response\""));

        let request = decode_server_message(
            r#"{"type":"lsp_hover_request","id":"hover-1","path":"src/lib.rs","line":3,"character":4}"#,
        )
        .unwrap();
        assert!(matches!(request, ServerMessage::LspHoverRequest { .. }));
    }

    #[test]
    fn suspension_and_handshake_decode() {
        assert!(matches!(
            decode_server_message(
                r#"{"type":"suspend","instruction":"check device","requires_feedback":true}"#
            )
            .unwrap(),
            ServerMessage::Suspend { .. }
        ));
        assert!(matches!(
            decode_server_message(r#"{"type":"handshake","plan":{"goal":"ship"}}"#).unwrap(),
            ServerMessage::Handshake { .. }
        ));
    }
}
