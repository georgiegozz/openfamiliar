//! Familiar Core — event bus, sessions, provider routing, workspace glue.

mod providers;

pub use providers::*;

use familiar_context::{ContextBudget, WorkspaceContext};
use familiar_permissions::{PermissionBroker, SecurityMode};
use familiar_storage::{AuditLog, Database};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MascotState {
    Idle,
    Listening,
    Thinking,
    Working,
    WaitingApproval,
    Success,
    Error,
    Sleeping,
    Offline,
}

impl MascotState {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "idle" => Some(Self::Idle),
            "listening" => Some(Self::Listening),
            "thinking" => Some(Self::Thinking),
            "working" => Some(Self::Working),
            "waiting_approval" | "approval" => Some(Self::WaitingApproval),
            "success" => Some(Self::Success),
            "error" => Some(Self::Error),
            "sleeping" => Some(Self::Sleeping),
            "offline" => Some(Self::Offline),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Listening => "listening",
            Self::Thinking => "thinking",
            Self::Working => "working",
            Self::WaitingApproval => "waiting_approval",
            Self::Success => "success",
            Self::Error => "error",
            Self::Sleeping => "sleeping",
            Self::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoreEvent {
    StateChanged {
        state: MascotState,
    },
    Speech {
        text: String,
    },
    Notification {
        message: String,
    },
    ApprovalQueued {
        id: String,
        title: String,
        detail: String,
    },
    ChatDelta {
        session_id: String,
        delta: String,
    },
    ChatDone {
        session_id: String,
    },
    ChatError {
        session_id: String,
        error: String,
    },
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Storage(#[from] familiar_storage::StorageError),
    #[error(transparent)]
    Context(#[from] familiar_context::ContextError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
}

pub type Result<T> = std::result::Result<T, CoreError>;

struct CoreInner {
    state: MascotState,
    mode: SecurityMode,
    speech: Option<String>,
    active_file: Option<String>,
    last_notification: Option<String>,
    approvals: Vec<(String, String, String)>,
    listeners: Vec<std::sync::mpsc::Sender<CoreEvent>>,
}

pub struct FamiliarCore {
    inner: RwLock<CoreInner>,
    pub db: Database,
    pub permissions: Arc<PermissionBroker>,
    workspace: RwLock<WorkspaceContext>,
    providers: ProviderRouter,
}

impl FamiliarCore {
    pub fn in_memory() -> Result<Self> {
        let db = Database::open_in_memory()?;
        let permissions = Arc::new(PermissionBroker::new(SecurityMode::Chat, None));
        Ok(Self {
            inner: RwLock::new(CoreInner {
                state: MascotState::Idle,
                mode: SecurityMode::Chat,
                speech: None,
                active_file: None,
                last_notification: None,
                approvals: Vec::new(),
                listeners: Vec::new(),
            }),
            db,
            permissions,
            workspace: RwLock::new(WorkspaceContext::new(ContextBudget::default())),
            providers: ProviderRouter::with_defaults(),
        })
    }

    pub fn open(db_path: &Path, audit_path: &Path) -> Result<Self> {
        let db = Database::open(db_path)?;
        let audit = AuditLog::new(audit_path);
        let permissions = Arc::new(PermissionBroker::new(SecurityMode::Chat, Some(audit)));
        Ok(Self {
            inner: RwLock::new(CoreInner {
                state: MascotState::Idle,
                mode: SecurityMode::Chat,
                speech: None,
                active_file: None,
                last_notification: None,
                approvals: Vec::new(),
                listeners: Vec::new(),
            }),
            db,
            permissions,
            workspace: RwLock::new(WorkspaceContext::new(ContextBudget::default())),
            providers: ProviderRouter::with_defaults(),
        })
    }

    pub fn subscribe(&self) -> std::sync::mpsc::Receiver<CoreEvent> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.inner.write().listeners.push(tx);
        rx
    }

    fn emit(&self, event: CoreEvent) {
        let mut inner = self.inner.write();
        inner.listeners.retain(|tx| tx.send(event.clone()).is_ok());
    }

    pub fn state(&self) -> MascotState {
        self.inner.read().state
    }

    pub fn set_state(&self, state: MascotState) {
        self.inner.write().state = state;
        self.emit(CoreEvent::StateChanged { state });
    }

    pub fn security_mode(&self) -> SecurityMode {
        self.inner.read().mode
    }

    pub fn set_security_mode(&self, mode: SecurityMode) {
        self.inner.write().mode = mode;
        self.permissions.set_mode(mode);
    }

    pub fn say(&self, text: &str) {
        self.inner.write().speech = Some(text.to_string());
        self.emit(CoreEvent::Speech {
            text: text.to_string(),
        });
    }

    pub fn notify(&self, message: &str) {
        self.inner.write().last_notification = Some(message.to_string());
        self.emit(CoreEvent::Notification {
            message: message.to_string(),
        });
    }

    pub fn queue_approval(&self, title: &str, detail: &str) -> String {
        let id = Uuid::new_v4().to_string();
        self.inner
            .write()
            .approvals
            .push((id.clone(), title.to_string(), detail.to_string()));
        self.set_state(MascotState::WaitingApproval);
        self.emit(CoreEvent::ApprovalQueued {
            id: id.clone(),
            title: title.to_string(),
            detail: detail.to_string(),
        });
        id
    }

    pub fn active_file(&self) -> Option<String> {
        self.inner.read().active_file.clone()
    }

    pub fn set_active_file(&self, path: Option<String>) {
        self.inner.write().active_file = path;
    }

    pub fn authorize_workspace(&self, id: &str, path: PathBuf) -> Result<()> {
        self.workspace.write().authorize_root(id, path)?;
        Ok(())
    }

    pub fn workspace_roots(&self) -> Vec<Value> {
        self.workspace
            .read()
            .roots()
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "path": r.path
                })
            })
            .collect()
    }

    pub fn preview_context(
        &self,
        workspace_id: &str,
        paths: &[String],
    ) -> std::result::Result<Value, String> {
        let rels: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let preview = self
            .workspace
            .read()
            .read_files(workspace_id, &rels, true)
            .map_err(|e| e.to_string())?;
        serde_json::to_value(preview).map_err(|e| e.to_string())
    }

    pub fn tree(&self, workspace_id: &str, max: usize) -> Result<Vec<String>> {
        Ok(self.workspace.read().summarize_tree(workspace_id, max)?)
    }

    pub fn providers(&self) -> &ProviderRouter {
        &self.providers
    }

    pub async fn chat_stream_collect(
        &self,
        provider_id: &str,
        model: &str,
        messages: Vec<ChatMessageDto>,
        budget_tokens: Option<u32>,
    ) -> Result<String> {
        self.set_state(MascotState::Thinking);
        let session = self.db.create_session("chat", provider_id, model)?;
        if let Some(last_user) = messages.iter().rev().find(|m| m.role == "user") {
            let _ = self
                .db
                .append_message(&session.id, "user", &last_user.content);
        }
        let req = ChatRequest {
            model: model.to_string(),
            messages,
            max_tokens: budget_tokens,
            session_id: session.id.clone(),
        };
        match self.providers.stream_collect(provider_id, req).await {
            Ok(text) => {
                let _ = self.db.append_message(&session.id, "assistant", &text);
                self.set_state(MascotState::Success);
                self.emit(CoreEvent::ChatDone {
                    session_id: session.id,
                });
                Ok(text)
            }
            Err(e) => {
                self.set_state(MascotState::Error);
                let msg = e.to_string();
                self.emit(CoreEvent::ChatError {
                    session_id: session.id,
                    error: msg.clone(),
                });
                Err(CoreError::Provider(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_transitions() {
        let core = FamiliarCore::in_memory().unwrap();
        assert_eq!(core.state(), MascotState::Idle);
        core.set_state(MascotState::Thinking);
        assert_eq!(core.state(), MascotState::Thinking);
    }

    #[tokio::test]
    async fn mock_provider_chat() {
        let core = FamiliarCore::in_memory().unwrap();
        let text = core
            .chat_stream_collect(
                "mock",
                "mock-model",
                vec![ChatMessageDto {
                    role: "user".into(),
                    content: "hola".into(),
                }],
                Some(128),
            )
            .await
            .unwrap();
        assert!(text.to_lowercase().contains("hola") || !text.is_empty());
    }

    #[test]
    fn mascot_state_parse_all_variants() {
        let cases = vec![
            ("idle", Some(MascotState::Idle)),
            ("listening", Some(MascotState::Listening)),
            ("thinking", Some(MascotState::Thinking)),
            ("working", Some(MascotState::Working)),
            ("waiting_approval", Some(MascotState::WaitingApproval)),
            ("approval", Some(MascotState::WaitingApproval)),
            ("success", Some(MascotState::Success)),
            ("error", Some(MascotState::Error)),
            ("sleeping", Some(MascotState::Sleeping)),
            ("offline", Some(MascotState::Offline)),
            ("invalid", None),
            ("", None),
        ];
        for (input, expected) in cases {
            assert_eq!(
                MascotState::parse(input),
                expected,
                "failed for input: {input}"
            );
        }
    }

    #[test]
    fn mascot_state_as_str_roundtrip() {
        let states = vec![
            MascotState::Idle,
            MascotState::Listening,
            MascotState::Thinking,
            MascotState::Working,
            MascotState::WaitingApproval,
            MascotState::Success,
            MascotState::Error,
            MascotState::Sleeping,
            MascotState::Offline,
        ];
        for state in states {
            let s = state.as_str();
            let parsed =
                MascotState::parse(s).unwrap_or_else(|| panic!("roundtrip failed for {s}"));
            assert_eq!(parsed, state);
        }
    }

    #[test]
    fn core_say_updates_speech() {
        let core = FamiliarCore::in_memory().unwrap();
        let rx = core.subscribe();
        core.say("hello perrito");
        let event = rx.try_recv().unwrap();
        match event {
            CoreEvent::Speech { text } => assert_eq!(text, "hello perrito"),
            other => panic!("expected Speech, got {:?}", other),
        }
    }

    #[test]
    fn core_notify_emits_event() {
        let core = FamiliarCore::in_memory().unwrap();
        let rx = core.subscribe();
        core.notify("build passed");
        let event = rx.try_recv().unwrap();
        match event {
            CoreEvent::Notification { message } => assert_eq!(message, "build passed"),
            other => panic!("expected Notification, got {:?}", other),
        }
    }

    #[test]
    fn core_queue_approval_transitions_state() {
        let core = FamiliarCore::in_memory().unwrap();
        let rx = core.subscribe();
        let id = core.queue_approval("Delete file", "rm -rf /tmp/test");
        assert!(!id.is_empty());
        assert_eq!(core.state(), MascotState::WaitingApproval);
        // Should emit StateChanged then ApprovalQueued
        let mut found_approval = false;
        while let Ok(event) = rx.try_recv() {
            if let CoreEvent::ApprovalQueued { title, detail, .. } = event {
                assert_eq!(title, "Delete file");
                assert_eq!(detail, "rm -rf /tmp/test");
                found_approval = true;
            }
        }
        assert!(found_approval, "ApprovalQueued event not emitted");
    }

    #[test]
    fn core_set_security_mode() {
        let core = FamiliarCore::in_memory().unwrap();
        assert_eq!(core.security_mode(), SecurityMode::Chat);
        core.set_security_mode(SecurityMode::Agent);
        assert_eq!(core.security_mode(), SecurityMode::Agent);
        assert_eq!(core.permissions.mode(), SecurityMode::Agent);
    }
}
