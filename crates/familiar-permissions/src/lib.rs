//! Permission Broker: no mutable op without explicit approval + audit.

use chrono::Utc;
use familiar_storage::{AuditEvent, AuditLog};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityMode {
    Chat,
    ReadOnly,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    WorkspaceRead,
    WorkspaceSearch,
    WorkspaceWrite,
    WorkspaceDelete,
    GitRead,
    GitDiff,
    ProcessExecute,
    NetworkFetch,
    ClipboardRead,
    ClipboardWrite,
}

impl Operation {
    pub fn is_mutable(self) -> bool {
        matches!(
            self,
            Operation::WorkspaceWrite
                | Operation::WorkspaceDelete
                | Operation::ProcessExecute
                | Operation::ClipboardWrite
        )
    }

    pub fn risk_hint(self) -> &'static str {
        match self {
            Operation::WorkspaceRead | Operation::WorkspaceSearch | Operation::GitRead | Operation::GitDiff => "low",
            Operation::ClipboardRead | Operation::NetworkFetch => "medium",
            Operation::WorkspaceWrite | Operation::ClipboardWrite => "high",
            Operation::WorkspaceDelete | Operation::ProcessExecute => "critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalScope {
    Once,
    Session,
    OperationInFolder,
    Deny,
    DenyAndBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub agent: String,
    pub operation: Operation,
    pub path: Option<String>,
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub files_to_modify: Vec<String>,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDecision {
    pub request_id: String,
    pub scope: ApprovalScope,
    pub approved: bool,
}

#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("operation not allowed in mode {0:?}")]
    ModeDenied(SecurityMode),
    #[error("approval required for {0:?}")]
    ApprovalRequired(Operation),
    #[error("blocked by rule: {0}")]
    Blocked(String),
    #[error("storage error: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, PermissionError>;

#[derive(Debug, Default)]
struct SessionGrants {
    ops: HashMap<Operation, bool>,
    folder_ops: HashMap<(Operation, String), bool>,
    blocks: HashMap<Operation, bool>,
}

pub struct PermissionBroker {
    mode: Mutex<SecurityMode>,
    grants: Mutex<SessionGrants>,
    pending: Mutex<HashMap<String, PermissionRequest>>,
    audit: Option<AuditLog>,
}

impl PermissionBroker {
    pub fn new(mode: SecurityMode, audit: Option<AuditLog>) -> Self {
        Self {
            mode: Mutex::new(mode),
            grants: Mutex::new(SessionGrants::default()),
            pending: Mutex::new(HashMap::new()),
            audit,
        }
    }

    pub fn set_mode(&self, mode: SecurityMode) {
        *self.mode.lock().expect("mode lock") = mode;
    }

    pub fn mode(&self) -> SecurityMode {
        *self.mode.lock().expect("mode lock")
    }

    pub fn request(&self, mut req: PermissionRequest) -> Result<PermissionRequest> {
        let mode = self.mode();
        match mode {
            SecurityMode::Chat => {
                return Err(PermissionError::ModeDenied(mode));
            }
            SecurityMode::ReadOnly => {
                if req.operation.is_mutable() {
                    return Err(PermissionError::ModeDenied(mode));
                }
                // read-only auto-allows non-mutable workspace/git reads
                if matches!(
                    req.operation,
                    Operation::WorkspaceRead
                        | Operation::WorkspaceSearch
                        | Operation::GitRead
                        | Operation::GitDiff
                ) {
                    self.audit_decision(&req, true, "read_only_auto");
                    return Ok(req);
                }
                return Err(PermissionError::ModeDenied(mode));
            }
            SecurityMode::Agent => {}
        }

        {
            let grants = self.grants.lock().expect("grants");
            if grants.blocks.get(&req.operation).copied().unwrap_or(false) {
                return Err(PermissionError::Blocked(format!("{:?}", req.operation)));
            }
            if grants.ops.get(&req.operation).copied().unwrap_or(false) {
                self.audit_decision(&req, true, "session_grant");
                return Ok(req);
            }
            if let Some(path) = &req.path {
                if grants
                    .folder_ops
                    .get(&(req.operation, path.clone()))
                    .copied()
                    .unwrap_or(false)
                {
                    self.audit_decision(&req, true, "folder_grant");
                    return Ok(req);
                }
            }
        }

        if req.id.is_empty() {
            req.id = Uuid::new_v4().to_string();
        }
        if req.risk.is_empty() {
            req.risk = req.operation.risk_hint().to_string();
        }
        self.pending
            .lock()
            .expect("pending")
            .insert(req.id.clone(), req.clone());
        Err(PermissionError::ApprovalRequired(req.operation))
    }

    pub fn decide(&self, decision: PermissionDecision) -> Result<()> {
        let mut pending = self.pending.lock().expect("pending");
        let req = pending
            .remove(&decision.request_id)
            .ok_or_else(|| PermissionError::Blocked("unknown request".into()))?;

        match decision.scope {
            ApprovalScope::Deny => {
                self.audit_decision(&req, false, "deny");
            }
            ApprovalScope::DenyAndBlock => {
                self.grants
                    .lock()
                    .expect("grants")
                    .blocks
                    .insert(req.operation, true);
                self.audit_decision(&req, false, "deny_and_block");
            }
            ApprovalScope::Once => {
                if !decision.approved {
                    self.audit_decision(&req, false, "deny");
                } else {
                    self.audit_decision(&req, true, "once");
                }
            }
            ApprovalScope::Session => {
                if decision.approved {
                    self.grants
                        .lock()
                        .expect("grants")
                        .ops
                        .insert(req.operation, true);
                    self.audit_decision(&req, true, "session");
                } else {
                    self.audit_decision(&req, false, "deny");
                }
            }
            ApprovalScope::OperationInFolder => {
                if decision.approved {
                    if let Some(path) = &req.path {
                        self.grants
                            .lock()
                            .expect("grants")
                            .folder_ops
                            .insert((req.operation, path.clone()), true);
                    }
                    self.audit_decision(&req, true, "folder");
                } else {
                    self.audit_decision(&req, false, "deny");
                }
            }
        }
        Ok(())
    }

    pub fn take_pending(&self, id: &str) -> Option<PermissionRequest> {
        self.pending.lock().expect("pending").get(id).cloned()
    }

    fn audit_decision(&self, req: &PermissionRequest, approved: bool, scope: &str) {
        if let Some(audit) = &self.audit {
            let _ = audit.append(&AuditEvent {
                id: Uuid::new_v4().to_string(),
                ts: Utc::now(),
                actor: req.agent.clone(),
                operation: format!("{:?}", req.operation),
                target: req.path.clone().unwrap_or_default(),
                decision: if approved {
                    format!("allow:{scope}")
                } else {
                    format!("deny:{scope}")
                },
                details: serde_json::json!({
                    "command": req.command,
                    "cwd": req.cwd,
                    "files": req.files_to_modify,
                    "risk": req.risk,
                }),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_mode_blocks_all_fs() {
        let broker = PermissionBroker::new(SecurityMode::Chat, None);
        let err = broker
            .request(PermissionRequest {
                id: String::new(),
                agent: "test".into(),
                operation: Operation::WorkspaceRead,
                path: Some("/tmp".into()),
                command: None,
                cwd: None,
                files_to_modify: vec![],
                risk: String::new(),
            })
            .unwrap_err();
        assert!(matches!(err, PermissionError::ModeDenied(SecurityMode::Chat)));
    }

    #[test]
    fn readonly_allows_read_blocks_write() {
        let broker = PermissionBroker::new(SecurityMode::ReadOnly, None);
        assert!(broker
            .request(PermissionRequest {
                id: String::new(),
                agent: "test".into(),
                operation: Operation::WorkspaceRead,
                path: Some("C:/proj".into()),
                command: None,
                cwd: None,
                files_to_modify: vec![],
                risk: String::new(),
            })
            .is_ok());
        assert!(broker
            .request(PermissionRequest {
                id: String::new(),
                agent: "test".into(),
                operation: Operation::WorkspaceWrite,
                path: Some("C:/proj".into()),
                command: None,
                cwd: None,
                files_to_modify: vec!["a.rs".into()],
                risk: String::new(),
            })
            .is_err());
    }

    #[test]
    fn agent_requires_then_session_grant() {
        let broker = PermissionBroker::new(SecurityMode::Agent, None);
        let req = PermissionRequest {
            id: "r1".into(),
            agent: "cli".into(),
            operation: Operation::ProcessExecute,
            path: Some("C:/ws".into()),
            command: Some("cargo test".into()),
            cwd: Some("C:/ws".into()),
            files_to_modify: vec![],
            risk: String::new(),
        };
        let err = broker.request(req).unwrap_err();
        assert!(matches!(err, PermissionError::ApprovalRequired(_)));
        broker
            .decide(PermissionDecision {
                request_id: "r1".into(),
                scope: ApprovalScope::Session,
                approved: true,
            })
            .unwrap();
        // second call allowed via session
        assert!(broker
            .request(PermissionRequest {
                id: String::new(),
                agent: "cli".into(),
                operation: Operation::ProcessExecute,
                path: Some("C:/ws".into()),
                command: Some("cargo test".into()),
                cwd: Some("C:/ws".into()),
                files_to_modify: vec![],
                risk: String::new(),
            })
            .is_ok());
    }
}