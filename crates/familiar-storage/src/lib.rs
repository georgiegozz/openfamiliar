//! Local persistence: SQLite sessions/history, JSONL audit, credential helpers.

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("paths unavailable")]
    PathsUnavailable,
    #[error("credential error: {0}")]
    Credential(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub audit_path: PathBuf,
    pub config_path: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from("dev", "OpenFamiliar", "OpenFamiliar")
            .ok_or(StorageError::PathsUnavailable)?;
        let data_dir = dirs.data_dir().to_path_buf();
        Ok(Self {
            db_path: data_dir.join("openfamiliar.db"),
            audit_path: data_dir.join("audit.jsonl"),
            config_path: data_dir.join("config.json"),
            data_dir,
        })
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub title: String,
    pub provider_id: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let db = Self { conn: Mutex::new(conn) };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn: Mutex::new(conn) };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS window_state (
                monitor_id TEXT PRIMARY KEY,
                x REAL NOT NULL,
                y REAL NOT NULL,
                scale REAL NOT NULL,
                mascot_size REAL NOT NULL
            );
            "#,
        )?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_version", [], |r| r.get(0))?;
        if count == 0 {
            conn
                .execute("INSERT INTO schema_version(version) VALUES (?1)", params![1i64])?;
        }
        Ok(())
    }

    pub fn create_session(&self, title: &str, provider_id: &str, model: &str) -> Result<SessionRecord> {
        let now = Utc::now();
        let rec = SessionRecord {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            provider_id: provider_id.to_string(),
            model: model.to_string(),
            created_at: now,
            updated_at: now,
        };
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO sessions(id, title, provider_id, model, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                rec.id,
                rec.title,
                rec.provider_id,
                rec.model,
                rec.created_at.to_rfc3339(),
                rec.updated_at.to_rfc3339()
            ],
        )?;
        Ok(rec)
    }

    pub fn append_message(&self, session_id: &str, role: &str, content: &str) -> Result<ChatMessage> {
        let msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
        };
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO messages(id, session_id, role, content, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![
                msg.id,
                session_id,
                msg.role,
                msg.content,
                msg.created_at.to_rfc3339()
            ],
        )?;
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), session_id],
        )?;
        Ok(msg)
    }

    pub fn list_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            let created: String = row.get(3)?;
            Ok(ChatMessage {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&created)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn save_window_state(&self, monitor_id: &str, x: f64, y: f64, scale: f64, mascot_size: f64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO window_state(monitor_id, x, y, scale, mascot_size) VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(monitor_id) DO UPDATE SET x=excluded.x, y=excluded.y, scale=excluded.scale, mascot_size=excluded.mascot_size",
            params![monitor_id, x, y, scale, mascot_size],
        )?;
        Ok(())
    }

    pub fn load_window_state(&self, monitor_id: &str) -> Result<Option<(f64, f64, f64, f64)>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT x, y, scale, mascot_size FROM window_state WHERE monitor_id = ?1",
        )?;
        let mut rows = stmt.query(params![monitor_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub ts: DateTime<Utc>,
    pub actor: String,
    pub operation: String,
    pub target: String,
    pub decision: String,
    pub details: serde_json::Value,
}

pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn append(&self, event: &AuditEvent) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        let line = serde_json::to_string(event)?;
        writeln!(file, "{line}")?;
        Ok(())
    }
}

pub struct CredentialStore {
    service: String,
}

impl CredentialStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    pub fn set_secret(&self, account: &str, secret: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, account)
            .map_err(|e| StorageError::Credential(e.to_string()))?;
        entry
            .set_password(secret)
            .map_err(|e| StorageError::Credential(e.to_string()))
    }

    pub fn get_secret(&self, account: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(&self.service, account)
            .map_err(|e| StorageError::Credential(e.to_string()))?;
        match entry.get_password() {
            Ok(p) => Ok(Some(p)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(StorageError::Credential(e.to_string())),
        }
    }

    pub fn delete_secret(&self, account: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, account)
            .map_err(|e| StorageError::Credential(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(StorageError::Credential(e.to_string())),
        }
    }
}

/// Redact common secret patterns for logs.
pub fn redact_secrets(input: &str) -> String {
    let patterns = [
        (r"(?i)(api[_-]?key|token|password|authorization)\s*[:=]\s*\S+", "$1=[REDACTED]"),
        (r"sk-[A-Za-z0-9]{10,}", "sk-[REDACTED]"),
        (r"AIza[0-9A-Za-z\-_]{10,}", "AIza[REDACTED]"),
    ];
    let mut out = input.to_string();
    for (pat, rep) in patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            out = re.replace_all(&out, rep).to_string();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_and_session_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        let s = db.create_session("t", "ollama-local", "llama3").unwrap();
        db.append_message(&s.id, "user", "hi").unwrap();
        let msgs = db.list_messages(&s.id).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "hi");
    }

    #[test]
    fn window_state_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        db.save_window_state("primary", 10.0, 20.0, 1.25, 128.0).unwrap();
        let st = db.load_window_state("primary").unwrap().unwrap();
        assert_eq!(st.0, 10.0);
        assert_eq!(st.3, 128.0);
    }

    #[test]
    fn redact_hides_keys() {
        let s = redact_secrets("api_key=sk-abcdefghijklmnop");
        assert!(!s.contains("sk-abcdefghijklmnop"));
        assert!(s.contains("REDACTED"));
    }

    #[test]
    fn session_multiple_messages_order() {
        let db = Database::open_in_memory().unwrap();
        let s = db.create_session("chat", "mock", "mock-model").unwrap();
        db.append_message(&s.id, "user", "first").unwrap();
        db.append_message(&s.id, "assistant", "second").unwrap();
        db.append_message(&s.id, "user", "third").unwrap();
        let msgs = db.list_messages(&s.id).unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].content, "first");
        assert_eq!(msgs[1].content, "second");
        assert_eq!(msgs[2].content, "third");
        assert_eq!(msgs[0].role, "user");
        assert_eq!(msgs[1].role, "assistant");
    }

    #[test]
    fn window_state_upsert() {
        let db = Database::open_in_memory().unwrap();
        db.save_window_state("primary", 10.0, 20.0, 1.0, 64.0).unwrap();
        db.save_window_state("primary", 100.0, 200.0, 2.0, 128.0).unwrap();
        let st = db.load_window_state("primary").unwrap().unwrap();
        assert_eq!(st.0, 100.0);
        assert_eq!(st.1, 200.0);
        assert_eq!(st.2, 2.0);
        assert_eq!(st.3, 128.0);
    }

    #[test]
    fn redact_hides_gemini_keys() {
        let s = redact_secrets("key=AIzaSyB1234567890abc");
        assert!(!s.contains("AIzaSyB1234567890abc"));
        assert!(s.contains("REDACTED"));
    }

    #[test]
    fn redact_preserves_safe_text() {
        let input = "Hello world! This is a normal log line with no secrets.";
        let output = redact_secrets(input);
        assert_eq!(input, output);
    }

    #[test]
    fn audit_log_writes_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-audit.jsonl");
        let log = AuditLog::new(&path);
        let event = AuditEvent {
            id: "ev1".into(),
            ts: chrono::Utc::now(),
            actor: "test-agent".into(),
            operation: "WorkspaceRead".into(),
            target: "/tmp/file.txt".into(),
            decision: "allow:once".into(),
            details: serde_json::json!({"risk": "low"}),
        };
        log.append(&event).unwrap();
        log.append(&event).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.trim().lines().collect();
        assert_eq!(lines.len(), 2);
        // Each line should be valid JSON
        for line in &lines {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(parsed["actor"], "test-agent");
        }
    }
}