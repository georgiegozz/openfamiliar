use crate::domain::{
    CommandError, ErrorKind, OneShotRequest, OneShotResult, ProviderStatus, MAX_PROMPT_CHARS,
    MAX_RESPONSE_BYTES, MAX_TIMEOUT_SECONDS, MIN_TIMEOUT_SECONDS,
};
use crate::services::logging::{SafeEvent, SafeLogger};
use familiar_oneshot_cli::{
    allowed_child_env, find_on_path, one_shot_args, prepare_workdir, resolve_configured_path,
    CliError, CliKind,
};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::timeout;
use uuid::Uuid;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone)]
struct ActiveRequest {
    pid: u32,
    cancelled: Arc<AtomicBool>,
}

pub struct CliAgentService {
    work_root: PathBuf,
    active: Mutex<HashMap<String, ActiveRequest>>,
    logger: SafeLogger,
}

impl CliAgentService {
    pub fn new(work_root: PathBuf, logger: SafeLogger) -> Self {
        Self {
            work_root,
            active: Mutex::new(HashMap::new()),
            logger,
        }
    }

    pub fn detect(&self, kind: CliKind) -> ProviderStatus {
        match find_on_path(kind) {
            Some(path) => {
                self.logger.event(SafeEvent::CodexDetected);
                ProviderStatus {
                    installed: true,
                    authenticated: true,
                    compatible: true,
                    version: None,
                    executable: path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(str::to_string),
                    message: detect_ready_message(kind),
                }
            }
            None => {
                self.logger.event(SafeEvent::CodexUnavailable);
                ProviderStatus {
                    installed: false,
                    authenticated: false,
                    compatible: false,
                    version: None,
                    executable: None,
                    message: missing_message(kind),
                }
            }
        }
    }

    pub async fn ask(
        &self,
        kind: CliKind,
        request: OneShotRequest,
        configured_path: Option<&str>,
    ) -> Result<OneShotResult, CommandError> {
        validate_request(&request)?;
        let program = resolve_program(kind, configured_path)?;
        let workdir = prepare_workdir(&self.work_root, kind).map_err(|_| {
            CommandError::new(
                ErrorKind::Unknown,
                "No se pudo preparar el directorio de trabajo vacío.",
            )
        })?;
        let args = one_shot_args(kind, request.prompt.trim()).map_err(map_cli_error)?;

        {
            let active = self.active.lock().await;
            if !active.is_empty() {
                return Err(CommandError::new(
                    ErrorKind::InvalidRequest,
                    "Ya existe una pregunta activa.",
                ));
            }
        }

        let mut command = Command::new(&program);
        command
            .args(&args)
            .current_dir(&workdir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.as_std_mut().creation_flags(CREATE_NO_WINDOW);
        }
        apply_child_env(&mut command);

        let mut child = command.spawn().map_err(|_| {
            CommandError::new(
                ErrorKind::CodexNotInstalled,
                format!("No se pudo iniciar {}.", display_name(kind)),
            )
        })?;
        let pid = child.id().ok_or_else(|| {
            CommandError::new(ErrorKind::Unknown, "El CLI no devolvió un process id.")
        })?;
        let cancelled = Arc::new(AtomicBool::new(false));
        self.active.lock().await.insert(
            request.request_id.clone(),
            ActiveRequest {
                pid,
                cancelled: cancelled.clone(),
            },
        );
        self.logger.event(SafeEvent::RequestStarted);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CommandError::new(ErrorKind::Unknown, "No se pudo capturar stdout."))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CommandError::new(ErrorKind::Unknown, "No se pudo capturar stderr."))?;
        let stdout_task = tokio::spawn(read_capped(stdout, MAX_RESPONSE_BYTES));
        let stderr_task = tokio::spawn(read_capped(stderr, MAX_RESPONSE_BYTES));

        let wait_result = timeout(Duration::from_secs(request.timeout_seconds), child.wait()).await;
        let status = match wait_result {
            Ok(Ok(status)) => status,
            Ok(Err(_)) => {
                self.active.lock().await.remove(&request.request_id);
                self.logger.event(SafeEvent::RequestFailed);
                return Err(CommandError::new(
                    ErrorKind::ProcessFailed,
                    format!("{} terminó de forma inesperada.", display_name(kind)),
                ));
            }
            Err(_) => {
                let _ = terminate_process_tree(pid).await;
                self.active.lock().await.remove(&request.request_id);
                self.logger.event(SafeEvent::RequestTimedOut);
                return Err(CommandError::new(
                    ErrorKind::Timeout,
                    "La pregunta superó el tiempo de espera.",
                ));
            }
        };

        let stdout = stdout_task.await.unwrap_or_default();
        let stderr = stderr_task.await.unwrap_or_default();
        self.active.lock().await.remove(&request.request_id);

        if cancelled.load(Ordering::SeqCst) {
            self.logger.event(SafeEvent::RequestCancelled);
            return Err(CommandError::new(
                ErrorKind::Cancelled,
                "La pregunta fue cancelada.",
            ));
        }
        if stdout.overflowed || stdout.invalid_utf8 {
            self.logger.event(SafeEvent::RequestFailed);
            return Err(CommandError::new(
                ErrorKind::OutputTooLarge,
                "La respuesta del CLI es inválida o demasiado grande.",
            ));
        }
        if !status.success() {
            self.logger.event(SafeEvent::RequestFailed);
            return Err(classify_process_error(kind, &stderr.text));
        }
        let answer = strip_ansi(&stdout.text).trim().to_string();
        if answer.is_empty() {
            self.logger.event(SafeEvent::RequestFailed);
            return Err(CommandError::new(
                ErrorKind::InvalidOutput,
                "El CLI no devolvió una respuesta en stdout.",
            ));
        }
        self.logger.event(SafeEvent::RequestSucceeded);
        Ok(OneShotResult {
            request_id: request.request_id,
            answer,
        })
    }

    pub async fn cancel(&self, request_id: &str) -> Result<(), CommandError> {
        let active = self.active.lock().await.get(request_id).cloned();
        let Some(active) = active else {
            return Ok(());
        };
        active.cancelled.store(true, Ordering::SeqCst);
        terminate_process_tree(active.pid)
            .await
            .map_err(|_| CommandError::new(ErrorKind::ProcessFailed, "No se pudo cancelar el CLI."))
    }
}

fn display_name(kind: CliKind) -> &'static str {
    match kind {
        CliKind::OpenCode => "OpenCode CLI",
        CliKind::Antigravity => "Antigravity CLI",
    }
}

fn missing_message(kind: CliKind) -> String {
    format!(
        "{} no está instalado. Se busca {} en PATH; no se ejecutan lanzadores .cmd o .bat.",
        display_name(kind),
        kind.program_name()
    )
}

fn detect_ready_message(kind: CliKind) -> String {
    match kind {
        CliKind::OpenCode => {
            "OpenCode CLI encontrado. OpenFamiliar no lee sus credenciales. Se lanza `opencode run` con la pregunta como un solo argumento, en un directorio temporal vacío, y con un `opencode.json` local que deniega herramientas. Residual: una regla global del operador podría prevalecer, y no hay flag oficial `--deny`.".to_string()
        }
        CliKind::Antigravity => {
            "Antigravity CLI encontrado. OpenFamiliar no lee sus credenciales. Se lanza `agy -p` con `--sandbox`, sin `--dangerously-skip-permissions`. En headless, las herramientas que pedirían aprobación se deniegan por omisión (soft-deny oficial). Residual: ajustes del operador en su propio CLI podrían permitir alguna acción.".to_string()
        }
    }
}

fn resolve_program(kind: CliKind, configured_path: Option<&str>) -> Result<PathBuf, CommandError> {
    if let Some(configured) = configured_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return resolve_configured_path(kind, configured).map_err(map_cli_error);
    }
    find_on_path(kind).ok_or_else(|| map_cli_error(CliError::Missing))
}

fn map_cli_error(error: CliError) -> CommandError {
    let message = match error {
        CliError::Missing => "El CLI no está instalado.".to_string(),
        CliError::ScriptWrapper => {
            "Se rechazó un lanzador .cmd o .bat. Se necesita el ejecutable nativo (.exe)."
                .to_string()
        }
        CliError::UnsafePath => {
            "La ruta configurada no es un ejecutable de CLI seguro.".to_string()
        }
        CliError::Workspace => "No se pudo preparar el directorio de trabajo vacío.".to_string(),
        CliError::PromptLooksLikeFlag => {
            "Se rechazó una pregunta que empieza con '-' (se interpretaría como una opción del CLI)."
                .to_string()
        }
    };
    let kind = match error {
        CliError::Workspace => ErrorKind::Unknown,
        CliError::PromptLooksLikeFlag => ErrorKind::InvalidRequest,
        _ => ErrorKind::CodexNotInstalled,
    };
    CommandError::new(kind, message)
}

fn validate_request(request: &OneShotRequest) -> Result<(), CommandError> {
    if Uuid::parse_str(&request.request_id).is_err() {
        return Err(CommandError::new(
            ErrorKind::InvalidRequest,
            "requestId debe ser un UUID válido.",
        ));
    }
    let prompt = request.prompt.trim();
    if prompt.is_empty() || prompt.chars().count() > MAX_PROMPT_CHARS {
        return Err(CommandError::new(
            ErrorKind::InvalidRequest,
            format!("La pregunta debe contener entre 1 y {MAX_PROMPT_CHARS} caracteres."),
        ));
    }
    if !(MIN_TIMEOUT_SECONDS..=MAX_TIMEOUT_SECONDS).contains(&request.timeout_seconds) {
        return Err(CommandError::new(
            ErrorKind::InvalidRequest,
            "El timeout debe estar entre 10 y 300 segundos.",
        ));
    }
    Ok(())
}

fn apply_child_env(command: &mut Command) {
    command.env_clear();
    for (key, value) in allowed_child_env() {
        command.env(key, value);
    }
}

#[derive(Debug, Default)]
struct CappedOutput {
    text: String,
    overflowed: bool,
    invalid_utf8: bool,
}

async fn read_capped<R>(mut reader: R, limit: usize) -> CappedOutput
where
    R: AsyncRead + Unpin,
{
    let mut bytes = Vec::with_capacity(limit.min(8_192));
    let mut buffer = [0_u8; 4_096];
    let mut overflowed = false;
    loop {
        match reader.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                let remaining = limit.saturating_sub(bytes.len());
                if read > remaining {
                    bytes.extend_from_slice(&buffer[..remaining]);
                    overflowed = true;
                } else if remaining > 0 {
                    bytes.extend_from_slice(&buffer[..read]);
                }
            }
        }
    }
    match String::from_utf8(bytes) {
        Ok(text) => CappedOutput {
            text,
            overflowed,
            invalid_utf8: false,
        },
        Err(_) => CappedOutput {
            text: String::new(),
            overflowed,
            invalid_utf8: true,
        },
    }
}

fn strip_ansi(value: &str) -> String {
    Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]")
        .map(|regex| regex.replace_all(value, "").to_string())
        .unwrap_or_else(|_| value.to_string())
}

fn classify_process_error(kind: CliKind, stderr: &str) -> CommandError {
    let normalized = stderr.to_ascii_lowercase();
    if normalized.contains("not logged in")
        || normalized.contains("authentication required")
        || normalized.contains("unauthorized")
        || normalized.contains("401")
    {
        CommandError::new(
            ErrorKind::NotAuthenticated,
            format!(
                "{} no tiene una sesión autenticada. OpenFamiliar no lee esas credenciales.",
                display_name(kind)
            ),
        )
    } else if normalized.contains("rate limit") || normalized.contains("429") {
        CommandError::new(
            ErrorKind::RateLimit,
            format!(
                "{} reportó un rate limit. Intenta más tarde.",
                display_name(kind)
            ),
        )
    } else {
        CommandError::new(
            ErrorKind::ProcessFailed,
            format!("{} terminó con un error.", display_name(kind)),
        )
    }
}

#[cfg(windows)]
async fn terminate_process_tree(pid: u32) -> Result<(), String> {
    let mut command = Command::new("taskkill.exe");
    command
        .arg("/PID")
        .arg(pid.to_string())
        .arg("/T")
        .arg("/F")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    use std::os::windows::process::CommandExt;
    command.as_std_mut().creation_flags(CREATE_NO_WINDOW);
    command
        .status()
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
async fn terminate_process_tree(_pid: u32) -> Result<(), String> {
    Err("OpenFamiliar v0.1 only supports Windows process-tree cancellation.".to_string())
}
