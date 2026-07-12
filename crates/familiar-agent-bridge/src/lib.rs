//! Agent Bridge Protocol: normalize CLI agent events; never steal credentials.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use familiar_permissions::{Operation, PermissionBroker, PermissionRequest, SecurityMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    SessionStarted { session_id: String, agent: String },
    Thinking { session_id: String, message: String },
    Reading { session_id: String, path: String },
    Editing { session_id: String, path: String },
    Running { session_id: String, command: String },
    WaitingApproval { session_id: String, request_id: String },
    Completed { session_id: String, summary: String },
    Failed { session_id: String, error: String },
    Cancelled { session_id: String },
    Log { session_id: String, line: String },
}

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("agent not available: {0}")]
    NotAvailable(String),
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("session not found")]
    SessionNotFound,
    #[error("cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, BridgeError>;

#[derive(Debug, Clone)]
pub struct AgentLaunch {
    pub agent_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: std::path::PathBuf,
    pub task: String,
}

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn is_available(&self) -> bool;
    fn build_command(&self, task: &str, cwd: &std::path::Path) -> Result<(String, Vec<String>)>;
}

pub struct GenericSubprocessAdapter {
    id: String,
    program: String,
    base_args: Vec<String>,
}

impl GenericSubprocessAdapter {
    pub fn new(id: impl Into<String>, program: impl Into<String>, base_args: Vec<String>) -> Self {
        Self {
            id: id.into(),
            program: program.into(),
            base_args,
        }
    }
}

#[async_trait]
impl AgentAdapter for GenericSubprocessAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn is_available(&self) -> bool {
        which_exists(&self.program)
    }

    fn build_command(&self, task: &str, _cwd: &std::path::Path) -> Result<(String, Vec<String>)> {
        let mut args = self.base_args.clone();
        args.push(task.to_string());
        Ok((self.program.clone(), args))
    }
}

pub struct NamedCliAdapter {
    id: &'static str,
    program: &'static str,
    arg_builder: fn(&str) -> Vec<String>,
}

impl NamedCliAdapter {
    pub const fn new(
        id: &'static str,
        program: &'static str,
        arg_builder: fn(&str) -> Vec<String>,
    ) -> Self {
        Self {
            id,
            program,
            arg_builder,
        }
    }
}

#[async_trait]
impl AgentAdapter for NamedCliAdapter {
    fn id(&self) -> &str {
        self.id
    }
    fn is_available(&self) -> bool {
        which_exists(self.program)
    }
    fn build_command(&self, task: &str, _cwd: &std::path::Path) -> Result<(String, Vec<String>)> {
        Ok((self.program.to_string(), (self.arg_builder)(task)))
    }
}

pub fn gemini_cli_adapter() -> NamedCliAdapter {
    NamedCliAdapter::new("gemini-cli", "gemini", |task| vec![task.to_string()])
}

pub fn codex_adapter() -> NamedCliAdapter {
    NamedCliAdapter::new("codex", "codex", |task| {
        vec!["exec".into(), task.to_string()]
    })
}

pub fn opencode_adapter() -> NamedCliAdapter {
    NamedCliAdapter::new("opencode", "opencode", |task| {
        vec!["run".into(), task.to_string()]
    })
}

pub fn antigravity_adapter() -> NamedCliAdapter {
    NamedCliAdapter::new("antigravity", "antigravity", |task| vec![task.to_string()])
}

struct LiveSession {
    child: Child,
    cancel: bool,
}

pub struct AgentBridge {
    adapters: HashMap<String, Arc<dyn AgentAdapter>>,
    sessions: Mutex<HashMap<String, LiveSession>>,
    permissions: Arc<PermissionBroker>,
}

impl AgentBridge {
    pub fn new(permissions: Arc<PermissionBroker>) -> Self {
        let mut adapters: HashMap<String, Arc<dyn AgentAdapter>> = HashMap::new();
        let list: Vec<Arc<dyn AgentAdapter>> = vec![
            Arc::new(gemini_cli_adapter()),
            Arc::new(codex_adapter()),
            Arc::new(opencode_adapter()),
            Arc::new(antigravity_adapter()),
            Arc::new(GenericSubprocessAdapter::new(
                "generic",
                "echo",
                vec![],
            )),
        ];
        for a in list {
            adapters.insert(a.id().to_string(), a);
        }
        Self {
            adapters,
            sessions: Mutex::new(HashMap::new()),
            permissions,
        }
    }

    pub fn list_adapters(&self) -> Vec<(String, bool)> {
        self.adapters
            .iter()
            .map(|(id, a)| (id.clone(), a.is_available()))
            .collect()
    }

    pub async fn start(
        &self,
        agent_id: &str,
        task: &str,
        cwd: std::path::PathBuf,
        tx: mpsc::Sender<AgentEvent>,
    ) -> Result<String> {
        let adapter = self
            .adapters
            .get(agent_id)
            .ok_or_else(|| BridgeError::NotAvailable(agent_id.into()))?
            .clone();
        if !adapter.is_available() {
            return Err(BridgeError::NotAvailable(agent_id.into()));
        }

        // Permission gate for process execution
        if self.permissions.mode() != SecurityMode::Agent {
            return Err(BridgeError::Permission(
                "agent mode required for CLI bridge".into(),
            ));
        }
        let (program, args) = adapter.build_command(task, &cwd)?;
        let req = PermissionRequest {
            id: Uuid::new_v4().to_string(),
            agent: agent_id.to_string(),
            operation: Operation::ProcessExecute,
            path: Some(cwd.display().to_string()),
            command: Some(format!("{program} {}", args.join(" "))),
            cwd: Some(cwd.display().to_string()),
            files_to_modify: vec![],
            risk: String::new(),
        };
        if let Err(e) = self.permissions.request(req.clone()) {
            let _ = tx
                .send(AgentEvent::WaitingApproval {
                    session_id: String::new(),
                    request_id: req.id.clone(),
                })
                .await;
            return Err(BridgeError::Permission(e.to_string()));
        }

        let session_id = Uuid::new_v4().to_string();
        let _ = tx
            .send(AgentEvent::SessionStarted {
                session_id: session_id.clone(),
                agent: agent_id.to_string(),
            })
            .await;

        let mut cmd = Command::new(&program);
        cmd.args(&args)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(
                session_id.clone(),
                LiveSession {
                    child,
                    cancel: false,
                },
            );
        }

        let sid = session_id.clone();
        if let Some(out) = stdout {
            let tx2 = tx.clone();
            let sid2 = sid.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(out).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx2
                        .send(AgentEvent::Log {
                            session_id: sid2.clone(),
                            line,
                        })
                        .await;
                }
            });
        }
        if let Some(err) = stderr {
            let tx2 = tx.clone();
            let sid2 = sid.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(err).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx2
                        .send(AgentEvent::Log {
                            session_id: sid2.clone(),
                            line,
                        })
                        .await;
                }
            });
        }

        let _ = tx
            .send(AgentEvent::Running {
                session_id: session_id.clone(),
                command: format!("{program} {}", args.join(" ")),
            })
            .await;

        Ok(session_id)
    }

    pub async fn cancel(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(mut live) = sessions.remove(session_id) {
            live.cancel = true;
            let _ = live.child.kill().await;
            Ok(())
        } else {
            Err(BridgeError::SessionNotFound)
        }
    }
}

fn which_exists(program: &str) -> bool {
    std::process::Command::new("where")
        .arg(program)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[allow(dead_code)]
fn now() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapters_registered() {
        let broker = Arc::new(PermissionBroker::new(SecurityMode::Chat, None));
        let bridge = AgentBridge::new(broker);
        let ids: Vec<_> = bridge.list_adapters().into_iter().map(|(i, _)| i).collect();
        assert!(ids.contains(&"generic".to_string()));
        assert!(ids.contains(&"codex".to_string()));
    }
}