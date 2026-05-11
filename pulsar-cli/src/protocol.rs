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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    StreamToken { content: String },
    ActionEvent { action: String, target: String },
    Error { message: String },
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
}
