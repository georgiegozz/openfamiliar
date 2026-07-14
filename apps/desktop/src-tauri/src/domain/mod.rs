use serde::{Deserialize, Serialize};

pub const MAX_PROMPT_CHARS: usize = 8_000;
pub const MAX_RESPONSE_BYTES: usize = 65_536;
pub const MIN_TIMEOUT_SECONDS: u64 = 10;
pub const MAX_TIMEOUT_SECONDS: u64 = 300;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub installed: bool,
    pub authenticated: bool,
    pub compatible: bool,
    pub version: Option<String>,
    pub executable: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneShotRequest {
    pub request_id: String,
    pub prompt: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneShotResult {
    pub request_id: String,
    pub answer: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    CodexNotInstalled,
    NotAuthenticated,
    IncompatibleVersion,
    RateLimit,
    Timeout,
    Cancelled,
    InvalidOutput,
    OutputTooLarge,
    ProcessFailed,
    InvalidRequest,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub kind: ErrorKind,
    pub message: String,
}

impl CommandError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppPreferences {
    pub mascot_position: Option<SavedPosition>,
    pub scale: u8,
    pub always_on_top: bool,
    pub click_through: bool,
    pub animations_enabled: bool,
    pub reduce_motion: bool,
    pub language: String,
    pub launch_at_startup: bool,
    pub codex_path: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            mascot_position: None,
            scale: 2,
            always_on_top: true,
            click_through: false,
            animations_enabled: true,
            reduce_motion: false,
            language: "es-MX".to_string(),
            launch_at_startup: false,
            codex_path: None,
            timeout_seconds: 120,
        }
    }
}

impl AppPreferences {
    pub fn validate(mut self) -> Self {
        self.scale = self.scale.clamp(1, 3);
        self.timeout_seconds = self
            .timeout_seconds
            .clamp(MIN_TIMEOUT_SECONDS, MAX_TIMEOUT_SECONDS);
        self.language = match self.language.as_str() {
            "en-US" => "en-US".to_string(),
            _ => "es-MX".to_string(),
        };
        self.codex_path = self.codex_path.and_then(|path| {
            let trimmed = path.trim();
            (!trimmed.is_empty() && trimmed.len() <= 1_024).then(|| trimmed.to_string())
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPosition {
    pub x: i32,
    pub y: i32,
    pub monitor_name: Option<String>,
    pub scale_factor: f64,
}
