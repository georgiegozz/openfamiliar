use crate::domain::{
    CommandError, ErrorKind, OneShotRequest, OneShotResult, ProviderStatus, MAX_PROMPT_CHARS,
    MAX_RESPONSE_BYTES, MAX_TIMEOUT_SECONDS, MIN_TIMEOUT_SECONDS,
};
use crate::services::logging::{SafeEvent, SafeLogger};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::timeout;
use uuid::Uuid;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
const PROBE_OUTPUT_LIMIT: usize = 32_768;
const ONE_SHOT_GUARD: &str = "Answer this independent one-shot question directly. Do not inspect local files, use workspace context, modify anything, or run commands.";

#[derive(Debug, Clone)]
struct ExecutableSpec {
    program: PathBuf,
    prefix_args: Vec<String>,
}

#[derive(Debug, Clone)]
struct ActiveRequest {
    pid: u32,
    cancelled: Arc<AtomicBool>,
}

pub struct CodexService {
    neutral_work_dir: PathBuf,
    active: Mutex<HashMap<String, ActiveRequest>>,
    logger: SafeLogger,
    fixed_spec: Option<ExecutableSpec>,
    detection_override: Option<ProviderStatus>,
    use_stdin_sentinel: bool,
}

impl CodexService {
    pub fn new(neutral_work_dir: PathBuf, logger: SafeLogger) -> Self {
        Self {
            neutral_work_dir,
            active: Mutex::new(HashMap::new()),
            logger,
            fixed_spec: None,
            detection_override: None,
            use_stdin_sentinel: true,
        }
    }

    #[cfg(test)]
    fn new_for_test(
        neutral_work_dir: PathBuf,
        logger: SafeLogger,
        program: PathBuf,
        prefix_args: Vec<String>,
        detection_override: Option<ProviderStatus>,
    ) -> Self {
        Self {
            neutral_work_dir,
            active: Mutex::new(HashMap::new()),
            logger,
            fixed_spec: Some(ExecutableSpec {
                program,
                prefix_args,
            }),
            detection_override,
            // Windows PowerShell's `-File` host rejects Codex's bare `-` argument.
            // The fixture still reads stdin; production always keeps the sentinel.
            use_stdin_sentinel: false,
        }
    }

    pub async fn detect(&self, configured_path: Option<&str>) -> ProviderStatus {
        if let Some(status) = &self.detection_override {
            return status.clone();
        }
        let spec = match self.resolve_executable(configured_path).await {
            Ok(spec) => spec,
            Err(message) => {
                self.logger.event(SafeEvent::CodexUnavailable);
                return ProviderStatus {
                    installed: false,
                    authenticated: false,
                    compatible: false,
                    version: None,
                    executable: None,
                    message,
                };
            }
        };

        let (version_probe, root_help, exec_help, login_status) = tokio::join!(
            self.run_probe(&spec, &["--version"]),
            self.run_probe(&spec, &["--help"]),
            self.run_probe(&spec, &["exec", "--help"]),
            self.run_probe(&spec, &["login", "status"]),
        );
        let version = match version_probe {
            Ok(output) if output.success => first_non_empty_line(&output.stdout),
            Ok(_) | Err(_) => None,
        };
        let compatible = match (&root_help, &exec_help) {
            (Ok(root), Ok(exec)) if root.success && exec.success => {
                root.stdout.contains("--ask-for-approval")
                    && exec.stdout.contains("--ephemeral")
                    && exec.stdout.contains("--sandbox")
                    && exec.stdout.contains("--skip-git-repo-check")
                    && exec.stdout.contains("--ignore-user-config")
                    && exec.stdout.contains("--ignore-rules")
            }
            _ => false,
        };
        let authenticated = login_status
            .map(|output| output.success)
            .unwrap_or(false);

        if compatible {
            self.logger.event(SafeEvent::CodexDetected);
        } else {
            self.logger.event(SafeEvent::CodexUnavailable);
        }
        ProviderStatus {
            installed: true,
            authenticated,
            compatible,
            version,
            executable: spec
                .program
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string),
            message: if !compatible {
                "La versión instalada no expone todos los controles seguros requeridos.".to_string()
            } else if !authenticated {
                "Codex CLI está instalado, pero la sesión no está autenticada.".to_string()
            } else {
                "Codex CLI está listo para preguntas independientes.".to_string()
            },
        }
    }

    pub async fn ask(
        &self,
        request: OneShotRequest,
        configured_path: Option<&str>,
    ) -> Result<OneShotResult, CommandError> {
        validate_request(&request)?;
        let spec = self
            .resolve_executable(configured_path)
            .await
            .map_err(|message| CommandError::new(ErrorKind::CodexNotInstalled, message))?;
        let status = self.detect(configured_path).await;
        if !status.compatible {
            return Err(CommandError::new(
                ErrorKind::IncompatibleVersion,
                status.message,
            ));
        }
        if !status.authenticated {
            return Err(CommandError::new(
                ErrorKind::NotAuthenticated,
                status.message,
            ));
        }

        {
            let active = self.active.lock().await;
            if !active.is_empty() {
                return Err(CommandError::new(
                    ErrorKind::InvalidRequest,
                    "Ya existe una pregunta activa.",
                ));
            }
        }

        let mut command = self.command_for(&spec);
        command
            .arg("--ask-for-approval")
            .arg("never")
            .arg("exec")
            .arg("--ephemeral")
            .arg("--sandbox")
            .arg("read-only")
            .arg("--skip-git-repo-check")
            .arg("--ignore-user-config")
            .arg("--ignore-rules")
            .current_dir(&self.neutral_work_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if self.use_stdin_sentinel {
            command.arg("-");
        }
        clear_sensitive_environment(&mut command);

        let mut child = command.spawn().map_err(|_| {
            CommandError::new(
                ErrorKind::CodexNotInstalled,
                "No se pudo iniciar Codex CLI.",
            )
        })?;
        let pid = child.id().ok_or_else(|| {
            CommandError::new(ErrorKind::Unknown, "Codex CLI no devolvió un process id.")
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

        let stdout = child.stdout.take().ok_or_else(|| {
            CommandError::new(ErrorKind::Unknown, "No se pudo capturar stdout.")
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            CommandError::new(ErrorKind::Unknown, "No se pudo capturar stderr.")
        })?;
        let stdout_task = tokio::spawn(read_capped(stdout, MAX_RESPONSE_BYTES));
        let stderr_task = tokio::spawn(read_capped(stderr, MAX_RESPONSE_BYTES));

        let guarded_prompt = format!("{ONE_SHOT_GUARD}\n\n{}", request.prompt.trim());
        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(guarded_prompt.as_bytes()).await.is_err() {
                let _ = terminate_process_tree(pid).await;
                self.active.lock().await.remove(&request.request_id);
                return Err(CommandError::new(
                    ErrorKind::ProcessFailed,
                    "No se pudo enviar la pregunta a Codex CLI.",
                ));
            }
            let _ = stdin.shutdown().await;
        }

        let wait_result = timeout(
            Duration::from_secs(request.timeout_seconds),
            child.wait(),
        )
        .await;
        let status = match wait_result {
            Ok(Ok(status)) => status,
            Ok(Err(_)) => {
                self.active.lock().await.remove(&request.request_id);
                self.logger.event(SafeEvent::RequestFailed);
                return Err(CommandError::new(
                    ErrorKind::ProcessFailed,
                    "Codex CLI terminó de forma inesperada.",
                ));
            }
            Err(_) => {
                let _ = terminate_process_tree(pid).await;
                let _ = timeout(Duration::from_secs(3), child.wait()).await;
                self.active.lock().await.remove(&request.request_id);
                self.logger.event(SafeEvent::RequestTimedOut);
                return Err(CommandError::new(
                    ErrorKind::Timeout,
                    "Codex CLI excedió el tiempo permitido.",
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
        if stdout.overflowed || stderr.overflowed {
            self.logger.event(SafeEvent::RequestFailed);
            return Err(CommandError::new(
                ErrorKind::OutputTooLarge,
                "La respuesta excedió el límite seguro de salida.",
            ));
        }
        if stdout.invalid_utf8 || stderr.invalid_utf8 {
            self.logger.event(SafeEvent::RequestFailed);
            return Err(CommandError::new(
                ErrorKind::InvalidOutput,
                "Codex CLI devolvió una salida de texto inválida.",
            ));
        }
        if !status.success() {
            self.logger.event(SafeEvent::RequestFailed);
            return Err(classify_process_error(&stderr.text));
        }

        let answer = strip_ansi(&stdout.text).trim().to_string();
        if answer.is_empty() {
            self.logger.event(SafeEvent::RequestFailed);
            return Err(CommandError::new(
                ErrorKind::InvalidOutput,
                "Codex CLI no devolvió una respuesta válida.",
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
        terminate_process_tree(active.pid).await.map_err(|_| {
            CommandError::new(
                ErrorKind::ProcessFailed,
                "No se pudo cancelar Codex CLI.",
            )
        })
    }

    pub async fn cancel_all(&self) {
        let active: Vec<ActiveRequest> = self.active.lock().await.values().cloned().collect();
        for request in active {
            request.cancelled.store(true, Ordering::SeqCst);
            let _ = terminate_process_tree(request.pid).await;
        }
    }

    async fn resolve_executable(
        &self,
        configured_path: Option<&str>,
    ) -> Result<ExecutableSpec, String> {
        if let Some(spec) = &self.fixed_spec {
            return spec
                .program
                .is_file()
                .then(|| spec.clone())
                .ok_or_else(|| "Codex CLI no está instalado.".to_string());
        }
        if let Some(value) = configured_path.map(str::trim).filter(|value| !value.is_empty()) {
            let path = PathBuf::from(value);
            if !is_safe_codex_executable(&path) {
                return Err("La ruta manual debe apuntar a un archivo codex.exe absoluto.".to_string());
            }
            return Ok(ExecutableSpec {
                program: path,
                prefix_args: Vec::new(),
            });
        }
        if let Some(path) = find_codex_on_path() {
            return Ok(ExecutableSpec {
                program: path,
                prefix_args: Vec::new(),
            });
        }
        Err("Codex CLI no está instalado o codex.exe no está disponible en PATH.".to_string())
    }

    async fn run_probe(
        &self,
        spec: &ExecutableSpec,
        args: &[&str],
    ) -> Result<ProbeOutput, String> {
        let mut command = self.command_for(spec);
        command
            .args(args)
            .current_dir(&self.neutral_work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        clear_sensitive_environment(&mut command);
        let mut child = command
            .spawn()
            .map_err(|_| "No se pudo iniciar Codex CLI durante la detección.".to_string())?;
        let pid = child
            .id()
            .ok_or_else(|| "Codex CLI no devolvió un process id durante la detección.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "No se pudo capturar la detección de Codex CLI.".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "No se pudo capturar la detección de Codex CLI.".to_string())?;
        let stdout_task = tokio::spawn(read_capped(stdout, PROBE_OUTPUT_LIMIT));
        let stderr_task = tokio::spawn(read_capped(stderr, PROBE_OUTPUT_LIMIT));
        let status = match timeout(PROBE_TIMEOUT, child.wait()).await {
            Ok(Ok(status)) => status,
            Ok(Err(_)) => return Err("Codex CLI falló durante la detección.".to_string()),
            Err(_) => {
                let _ = terminate_process_tree(pid).await;
                let _ = timeout(Duration::from_secs(2), child.wait()).await;
                return Err("Codex CLI no respondió durante la detección.".to_string());
            }
        };
        let stdout = stdout_task.await.unwrap_or_default();
        let stderr = stderr_task.await.unwrap_or_default();
        if stdout.overflowed
            || stderr.overflowed
            || stdout.invalid_utf8
            || stderr.invalid_utf8
        {
            return Err("Codex CLI devolvió una salida de detección inválida.".to_string());
        }
        Ok(ProbeOutput {
            success: status.success(),
            stdout: strip_ansi(&stdout.text),
            stderr: strip_ansi(&stderr.text),
        })
    }

    fn command_for(&self, spec: &ExecutableSpec) -> Command {
        let mut command = Command::new(&spec.program);
        command.args(&spec.prefix_args);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command
                .as_std_mut()
                .creation_flags(CREATE_NO_WINDOW);
        }
        command
    }
}

#[derive(Debug)]
struct ProbeOutput {
    success: bool,
    stdout: String,
    #[allow(dead_code)]
    stderr: String,
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

fn is_safe_codex_executable(path: &Path) -> bool {
    path.is_absolute()
        && path.is_file()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case("codex.exe"))
            .unwrap_or(false)
}

fn clear_sensitive_environment(command: &mut Command) {
    for variable in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "AZURE_OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "GOOGLE_API_KEY",
        "GEMINI_API_KEY",
        "OPENROUTER_API_KEY",
        "XAI_API_KEY",
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
    ] {
        command.env_remove(variable);
    }
}

fn find_codex_on_path() -> Option<PathBuf> {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .map(|directory| directory.join("codex.exe"))
        .find(|candidate| candidate.is_file())
}

fn first_non_empty_line(value: &str) -> Option<String> {
    value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

fn strip_ansi(value: &str) -> String {
    Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]")
        .map(|regex| regex.replace_all(value, "").to_string())
        .unwrap_or_else(|_| value.to_string())
}

fn classify_process_error(stderr: &str) -> CommandError {
    let normalized = stderr.to_ascii_lowercase();
    if normalized.contains("not logged in")
        || normalized.contains("authentication required")
        || normalized.contains("unauthorized")
        || normalized.contains("401")
    {
        CommandError::new(
            ErrorKind::NotAuthenticated,
            "Codex CLI no tiene una sesión autenticada.",
        )
    } else if normalized.contains("rate limit") || normalized.contains("429") {
        CommandError::new(
            ErrorKind::RateLimit,
            "Codex CLI reportó un rate limit. Intenta más tarde.",
        )
    } else {
        CommandError::new(
            ErrorKind::ProcessFailed,
            "Codex CLI terminó con un error.",
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
    command
        .as_std_mut()
        .creation_flags(CREATE_NO_WINDOW);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use tempfile::TempDir;

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn ready_status(authenticated: bool) -> ProviderStatus {
        ProviderStatus {
            installed: true,
            authenticated,
            compatible: true,
            version: Some("codex-cli 99.0.0-fake".into()),
            executable: Some("codex.exe".into()),
            message: if authenticated {
                "Codex CLI está listo para preguntas independientes.".into()
            } else {
                "Codex CLI está instalado, pero la sesión no está autenticada.".into()
            },
        }
    }

    fn fixture_service_with_probes(
        scenario: &str,
        probe_detection: bool,
    ) -> (TempDir, Arc<CodexService>) {
        let temp = tempfile::tempdir().expect("tempdir");
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("fake-codex.ps1");
        let logger = SafeLogger::new(temp.path().join("logs"));
        std::fs::create_dir_all(temp.path().join("logs")).expect("logs");
        let service = CodexService::new_for_test(
            temp.path().to_path_buf(),
            logger,
            PathBuf::from(std::env::var_os("SystemRoot").expect("SystemRoot"))
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe"),
            vec![
                "-NoProfile".into(),
                "-ExecutionPolicy".into(),
                "Bypass".into(),
                "-File".into(),
                script.display().to_string(),
                "-Scenario".into(),
                scenario.into(),
            ],
            (!probe_detection).then(|| ready_status(scenario != "unauthenticated")),
        );
        (temp, Arc::new(service))
    }

    fn fixture_service(scenario: &str) -> (TempDir, Arc<CodexService>) {
        fixture_service_with_probes(scenario, false)
    }

    fn request(timeout_seconds: u64) -> OneShotRequest {
        OneShotRequest {
            request_id: Uuid::new_v4().to_string(),
            prompt: "¿Cuál es la capital de México?".into(),
            timeout_seconds,
        }
    }

    #[tokio::test]
    async fn detect_success() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service_with_probes("success", true);
        let status = service.detect(None).await;
        assert!(status.installed);
        assert!(status.authenticated);
        assert!(status.compatible);
    }

    #[tokio::test]
    async fn executable_missing() {
        let _guard = test_lock().lock().await;
        let temp = tempfile::tempdir().expect("tempdir");
        let logger = SafeLogger::new(temp.path().join("logs"));
        let service = CodexService::new_for_test(
            temp.path().to_path_buf(),
            logger,
            temp.path().join("missing.exe"),
            Vec::new(),
            None,
        );
        let status = service.detect(None).await;
        assert!(!status.installed);
        assert!(!status.compatible);
    }

    #[tokio::test]
    async fn successful_unicode_response() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("unicode");
        let result = service.ask(request(10), None).await.expect("answer");
        assert_eq!(result.answer, "¡Respuesta rápida! 🐶");
    }

    #[tokio::test]
    async fn strips_ansi_output() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("ansi");
        let result = service.ask(request(10), None).await.expect("answer");
        assert_eq!(result.answer, "green answer");
    }

    #[tokio::test]
    async fn successful_stdout_ignores_nonfatal_stderr() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("mixed");
        let result = service.ask(request(10), None).await.expect("answer");
        assert_eq!(result.answer, "safe stdout answer");
    }

    #[tokio::test]
    async fn rejects_invalid_utf8_output() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("invalid-utf8");
        let error = service.ask(request(10), None).await.expect_err("error");
        assert_eq!(error.kind, ErrorKind::InvalidOutput);
    }

    #[tokio::test]
    async fn classifies_unauthenticated() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("unauthenticated");
        let error = service.ask(request(10), None).await.expect_err("error");
        assert_eq!(error.kind, ErrorKind::NotAuthenticated);
    }

    #[tokio::test]
    async fn classifies_rate_limit() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("rate-limit");
        let error = service.ask(request(10), None).await.expect_err("error");
        assert_eq!(error.kind, ErrorKind::RateLimit);
    }

    #[tokio::test]
    async fn rejects_oversized_output() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("large");
        let error = service.ask(request(10), None).await.expect_err("error");
        assert_eq!(error.kind, ErrorKind::OutputTooLarge);
    }

    #[tokio::test]
    async fn classifies_nonzero_exit_without_exposing_stderr() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("nonzero");
        let error = service.ask(request(10), None).await.expect_err("error");
        assert_eq!(error.kind, ErrorKind::ProcessFailed);
        assert!(!error.message.contains("process failed"));
    }

    #[tokio::test]
    async fn timeout_terminates_process_tree() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("slow");
        let mut req = request(10);
        req.timeout_seconds = 10;
        let future = service.ask(req.clone(), None);
        let result = timeout(Duration::from_secs(25), future)
            .await
            .expect("outer timeout");
        assert_eq!(result.expect_err("timeout").kind, ErrorKind::Timeout);
    }

    #[tokio::test]
    async fn cancellation_terminates_process_tree() {
        let _guard = test_lock().lock().await;
        let (_temp, service) = fixture_service("child");
        let req = request(30);
        let request_id = req.request_id.clone();
        let running = {
            let service = service.clone();
            tokio::spawn(async move { service.ask(req, None).await })
        };
        timeout(Duration::from_secs(10), async {
            loop {
                if service.active.lock().await.contains_key(&request_id) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .expect("request starts");
        service.cancel(&request_id).await.expect("cancel");
        let error = running.await.expect("join").expect_err("cancelled");
        assert_eq!(error.kind, ErrorKind::Cancelled);
    }

    #[test]
    fn manual_executable_must_be_named_codex_exe() {
        let temp = tempfile::tempdir().expect("tempdir");
        let codex = temp.path().join("codex.exe");
        let other = temp.path().join("other.exe");
        std::fs::write(&codex, b"fixture").expect("codex");
        std::fs::write(&other, b"fixture").expect("other");
        assert!(is_safe_codex_executable(&codex));
        assert!(!is_safe_codex_executable(&other));
    }
}
