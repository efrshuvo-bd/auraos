//! Length-prefixed JSON IPC for AuraOS userspace services.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("io error: {0}")]
    Io(String),
    #[error("invalid frame")]
    InvalidFrame,
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Hello {
        from: String,
        version: String,
    },
    HelloAck {
        from: String,
    },
    AgentRequest {
        id: u64,
        prompt: String,
    },
    AgentResponse {
        id: u64,
        text: String,
        tools_used: Vec<String>,
    },
    ToolCall {
        id: u64,
        name: String,
        args: serde_json::Value,
    },
    ToolResult {
        id: u64,
        ok: bool,
        output: String,
    },
    SystemEvent {
        kind: String,
        detail: String,
    },
    Error {
        id: Option<u64>,
        message: String,
    },
}

/// Encode a message as `[u32 LE length][utf8 json]`.
pub fn encode(msg: &Message) -> Result<Vec<u8>, IpcError> {
    let json = serde_json::to_vec(msg)?;
    let mut out = Vec::with_capacity(4 + json.len());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(&json);
    Ok(out)
}

/// Decode one frame from a buffer; returns (message, bytes_consumed).
pub fn try_decode(buf: &[u8]) -> Result<Option<(Message, usize)>, IpcError> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if buf.len() < 4 + len {
        return Ok(None);
    }
    let msg: Message = serde_json::from_slice(&buf[4..4 + len])?;
    Ok(Some((msg, 4 + len)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let msg = Message::AgentRequest {
            id: 1,
            prompt: "help".into(),
        };
        let bytes = encode(&msg).unwrap();
        let (decoded, n) = try_decode(&bytes).unwrap().unwrap();
        assert_eq!(n, bytes.len());
        match decoded {
            Message::AgentRequest { id, prompt } => {
                assert_eq!(id, 1);
                assert_eq!(prompt, "help");
            }
            _ => panic!("wrong variant"),
        }
    }
}
