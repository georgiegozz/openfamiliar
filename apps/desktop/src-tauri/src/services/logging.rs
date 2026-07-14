use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOG_BYTES: u64 = 1_048_576;

#[derive(Debug, Clone, Copy)]
pub enum SafeEvent {
    AppStarted,
    AppStopped,
    CodexDetected,
    CodexUnavailable,
    RequestStarted,
    RequestSucceeded,
    RequestFailed,
    RequestCancelled,
    RequestTimedOut,
}

impl SafeEvent {
    fn as_str(self) -> &'static str {
        match self {
            Self::AppStarted => "app_started",
            Self::AppStopped => "app_stopped",
            Self::CodexDetected => "codex_detected",
            Self::CodexUnavailable => "codex_unavailable",
            Self::RequestStarted => "request_started",
            Self::RequestSucceeded => "request_succeeded",
            Self::RequestFailed => "request_failed",
            Self::RequestCancelled => "request_cancelled",
            Self::RequestTimedOut => "request_timed_out",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SafeLogger {
    path: PathBuf,
}

impl SafeLogger {
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            path: log_dir.join("openfamiliar.log"),
        }
    }

    pub fn event(&self, event: SafeEvent) {
        if self
            .path
            .metadata()
            .map(|metadata| metadata.len() > MAX_LOG_BYTES)
            .unwrap_or(false)
        {
            let previous = self.path.with_extension("log.previous");
            let _ = fs::remove_file(&previous);
            let _ = fs::rename(&self.path, previous);
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(file, "{timestamp} {}", event.as_str());
        }
    }
}
