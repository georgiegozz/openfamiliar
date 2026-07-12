//! Local MCP-compatible tool/resource catalog (stdio-oriented, no public bind).

use familiar_core::{FamiliarCore, MascotState};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, McpError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct LocalMcpServer {
    core: Arc<FamiliarCore>,
}

impl LocalMcpServer {
    pub fn new(core: Arc<FamiliarCore>) -> Self {
        Self { core }
    }

    pub fn list_resources(&self) -> Vec<McpResource> {
        vec![
            McpResource {
                uri: "familiar://workspace/tree".into(),
                name: "workspace_tree".into(),
                description: "Summarized workspace tree".into(),
            },
            McpResource {
                uri: "familiar://workspace/active-file".into(),
                name: "active_file".into(),
                description: "Active file context if set".into(),
            },
            McpResource {
                uri: "familiar://workspace/git-diff".into(),
                name: "git_diff".into(),
                description: "Git status/diff summary".into(),
            },
            McpResource {
                uri: "familiar://session/status".into(),
                name: "session_status".into(),
                description: "Current mascot/session status".into(),
            },
        ]
    }

    pub fn list_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "familiar_set_state".into(),
                description: "Set mascot visual state".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": { "state": { "type": "string" } },
                    "required": ["state"]
                }),
            },
            McpTool {
                name: "familiar_say".into(),
                description: "Show a speech bubble message".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": { "text": { "type": "string" } },
                    "required": ["text"]
                }),
            },
            McpTool {
                name: "familiar_request_approval".into(),
                description: "Surface an approval request on the desktop UI".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "detail": { "type": "string" }
                    },
                    "required": ["title"]
                }),
            },
            McpTool {
                name: "familiar_notify".into(),
                description: "Local notification".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": { "message": { "type": "string" } },
                    "required": ["message"]
                }),
            },
            McpTool {
                name: "familiar_get_context".into(),
                description: "Get context preview for authorized workspace".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": { "type": "string" },
                        "paths": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["workspace_id"]
                }),
            },
        ]
    }

    pub fn read_resource(&self, uri: &str) -> Result<Value> {
        match uri {
            "familiar://session/status" => Ok(json!({
                "state": self.core.state(),
                "mode": self.core.security_mode(),
            })),
            "familiar://workspace/tree" => {
                let roots = self.core.workspace_roots();
                Ok(json!({ "roots": roots }))
            }
            "familiar://workspace/git-diff" => Ok(json!({
                "note": "Use familiar_get_context / core API with a workspace selection"
            })),
            "familiar://workspace/active-file" => Ok(json!({
                "active_file": self.core.active_file()
            })),
            other => Err(McpError::Message(format!("unknown resource {other}"))),
        }
    }

    pub fn call_tool(&self, name: &str, args: Value) -> Result<Value> {
        match name {
            "familiar_set_state" => {
                let state = args
                    .get("state")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::Message("state required".into()))?;
                let parsed = MascotState::parse(state)
                    .ok_or_else(|| McpError::Message(format!("invalid state {state}")))?;
                self.core.set_state(parsed);
                Ok(json!({ "ok": true, "state": state }))
            }
            "familiar_say" => {
                let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
                self.core.say(text);
                Ok(json!({ "ok": true }))
            }
            "familiar_request_approval" => {
                let title = args
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Approval");
                let detail = args.get("detail").and_then(|v| v.as_str()).unwrap_or("");
                let id = self.core.queue_approval(title, detail);
                Ok(json!({ "ok": true, "request_id": id }))
            }
            "familiar_notify" => {
                let message = args.get("message").and_then(|v| v.as_str()).unwrap_or("");
                self.core.notify(message);
                Ok(json!({ "ok": true }))
            }
            "familiar_get_context" => {
                let workspace_id = args
                    .get("workspace_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::Message("workspace_id required".into()))?;
                let paths: Vec<String> = args
                    .get("paths")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let preview = self
                    .core
                    .preview_context(workspace_id, &paths)
                    .map_err(McpError::Message)?;
                Ok(preview)
            }
            other => Err(McpError::Message(format!("unknown tool {other}"))),
        }
    }

    pub fn handle_jsonrpc(&self, method: &str, params: Value) -> Result<Value> {
        match method {
            "resources/list" => Ok(json!(self.list_resources())),
            "tools/list" => Ok(json!(self.list_tools())),
            "resources/read" => {
                let uri = params
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::Message("uri required".into()))?;
                self.read_resource(uri)
            }
            "tools/call" => {
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::Message("name required".into()))?;
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                self.call_tool(name, args)
            }
            other => Err(McpError::Message(format!("unsupported method {other}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_tools_and_sets_state() {
        let core = Arc::new(FamiliarCore::in_memory().unwrap());
        let mcp = LocalMcpServer::new(core.clone());
        assert!(mcp
            .list_tools()
            .iter()
            .any(|t| t.name == "familiar_set_state"));
        let res = mcp
            .call_tool("familiar_set_state", json!({"state": "thinking"}))
            .unwrap();
        assert_eq!(res["ok"], true);
        assert_eq!(core.state(), MascotState::Thinking);
    }
}
