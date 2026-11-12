//! One-shot argv and executable helpers for OpenCode and Antigravity.
//!
//! Official sources consulted on 12 Nov 2026 (America/Mexico_City):
//! - OpenCode: https://opencode.ai/docs/cli/commands/
//!   `opencode run "<prompt>"` — the prompt is a positional argument.
//!   `--auto` auto-approves non-denied permissions (dangerous); we never pass it.
//!   `--password` uses the short flag `-p`; the prompt is therefore never passed
//!   with `-p`. There is no official `--deny` CLI flag; we write a project-local
//!   `opencode.json` in an empty temp cwd (see [`OPENCODE_DENY_CONFIG`]).
//! - Antigravity: https://antigravity.google/docs/cli/headless/
//!   `agy -p "<prompt>"` (`--print` / `--prompt` aliases). We use `-p` and pass
//!   the prompt as the next, separate argument. We add official `--sandbox` and
//!   never pass `--dangerously-skip-permissions`. In headless mode, tools that
//!   would need an interactive approval are soft-denied by default.
//!
//! This crate never reads the CLIs' credential stores (`auth.json`,
//! `~/.opencode`, `~/.gemini/antigravity-cli`, etc.). Commands are built as
//! argv arrays for `Command::new(program)` — no shell, no `cmd /c`.

use std::fs;
use std::path::{Path, PathBuf};

pub const SPECIAL_PROMPT_FIXTURE: &str = r#"reply with: " & | % and -p"#;

/// Project-local OpenCode permission policy written into the empty temp cwd.
/// Official `opencode run` has no deny-tools flag
/// (https://opencode.ai/docs/cli/commands/). Permissions are configured in
/// `opencode.json` (https://opencode.ai/docs/permissions/).
///
/// Residual: OpenCode loads global and agent rules after project rules and the
/// last match wins, so a user global allow could theoretically override this
/// file. We do not read or write the operator's global config (that would
/// approach their credential store). We also never pass `--auto`. Combined with
/// an empty cwd, tools have nothing useful to touch.
pub const OPENCODE_DENY_CONFIG: &str = r#"{
  "$schema": "https://opencode.ai/config.json",
  "permissions": [
    { "action": "*", "resource": "*", "effect": "deny" }
  ]
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliKind {
    OpenCode,
    Antigravity,
}

impl CliKind {
    pub fn program_name(self) -> &'static str {
        match self {
            Self::OpenCode => "opencode",
            Self::Antigravity => "agy",
        }
    }

    pub fn from_provider_id(id: &str) -> Option<Self> {
        match id {
            "opencode" => Some(Self::OpenCode),
            "agy" => Some(Self::Antigravity),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CliError {
    #[error("CLI executable is missing")]
    Missing,
    #[error("refusing to launch a .cmd or .bat wrapper; a real .exe is required")]
    ScriptWrapper,
    #[error("the configured path is not a safe CLI executable")]
    UnsafePath,
    #[error("could not prepare the empty working directory")]
    Workspace,
}

/// Build argv (without the program name) for a one-shot question.
/// The prompt is always a single argument, including when it contains
/// `"`, `&`, `|`, `%`, or `-p`.
pub fn one_shot_args(kind: CliKind, prompt: &str) -> Vec<String> {
    match kind {
        CliKind::OpenCode => vec!["run".to_string(), prompt.to_string()],
        CliKind::Antigravity => vec![
            "--sandbox".to_string(),
            "-p".to_string(),
            prompt.to_string(),
        ],
    }
}

pub fn args_are_safe(args: &[String]) -> bool {
    !args.iter().any(|arg| {
        arg == "--auto"
            || arg == "--dangerously-skip-permissions"
            || arg == "--yolo"
            || arg.eq_ignore_ascii_case("/c")
            || arg.eq_ignore_ascii_case("cmd")
            || arg.eq_ignore_ascii_case("cmd.exe")
    })
}

pub fn is_script_wrapper(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat"))
        .unwrap_or(false)
}

fn expected_file_names(kind: CliKind) -> &'static [&'static str] {
    match kind {
        CliKind::OpenCode => &["opencode", "opencode.exe"],
        CliKind::Antigravity => &["agy", "agy.exe"],
    }
}

pub fn is_allowed_program_name(kind: CliKind, path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            expected_file_names(kind)
                .iter()
                .any(|expected| name.eq_ignore_ascii_case(expected))
        })
        .unwrap_or(false)
}

/// If `path` is a .cmd/.bat wrapper, try to resolve a sibling real executable.
/// Never returns the wrapper itself. Never invokes `cmd /c`.
pub fn resolve_real_executable(kind: CliKind, path: &Path) -> Result<PathBuf, CliError> {
    if is_script_wrapper(path) {
        if let Some(parent) = path.parent() {
            for name in expected_file_names(kind) {
                let candidate = parent.join(name);
                if candidate.is_file() && !is_script_wrapper(&candidate) {
                    return Ok(candidate);
                }
            }
        }
        return Err(CliError::ScriptWrapper);
    }
    if !path.is_file() {
        return Err(CliError::Missing);
    }
    if !is_allowed_program_name(kind, path) {
        return Err(CliError::UnsafePath);
    }
    Ok(path.to_path_buf())
}

pub fn resolve_configured_path(kind: CliKind, configured: &str) -> Result<PathBuf, CliError> {
    let trimmed = configured.trim();
    if trimmed.is_empty() {
        return Err(CliError::Missing);
    }
    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err(CliError::UnsafePath);
    }
    resolve_real_executable(kind, &path)
}

pub fn find_on_path(kind: CliKind) -> Option<PathBuf> {
    let directories = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>());
    find_in_directories(kind, directories)
}

pub fn find_in_directories<I>(kind: CliKind, directories: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = PathBuf>,
{
    for directory in directories {
        for name in expected_file_names(kind) {
            let candidate = directory.join(name);
            if candidate.is_file()
                && !is_script_wrapper(&candidate)
                && is_allowed_program_name(kind, &candidate)
            {
                return Some(candidate);
            }
        }
        for wrapper in ["cmd", "bat"] {
            let wrapper_path = directory.join(format!("{}.{wrapper}", kind.program_name()));
            if wrapper_path.is_file() {
                if let Ok(resolved) = resolve_real_executable(kind, &wrapper_path) {
                    return Some(resolved);
                }
            }
        }
    }
    None
}

/// Create (or reuse) an empty working directory and write the OpenCode deny
/// policy when needed. Does not read files from the user profile.
pub fn prepare_workdir(root: &Path, kind: CliKind) -> Result<PathBuf, CliError> {
    fs::create_dir_all(root).map_err(|_| CliError::Workspace)?;
    let dir = root.join(kind.program_name());
    if dir.exists() {
        // Keep the workspace empty: remove leftover files, not the directory.
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let _ = fs::remove_dir_all(&path);
                } else {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    } else {
        fs::create_dir_all(&dir).map_err(|_| CliError::Workspace)?;
    }
    if kind == CliKind::OpenCode {
        fs::write(dir.join("opencode.json"), OPENCODE_DENY_CONFIG)
            .map_err(|_| CliError::Workspace)?;
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn opencode_prompt_with_special_chars_is_a_single_argument() {
        let args = one_shot_args(CliKind::OpenCode, SPECIAL_PROMPT_FIXTURE);
        assert_eq!(args[0], "run");
        assert_eq!(args[1], SPECIAL_PROMPT_FIXTURE);
        assert_eq!(args.len(), 2);
        assert!(args_are_safe(&args));
        assert!(!args.contains(&"-p".to_string()));
        assert!(!args.contains(&"--auto".to_string()));
        assert!(!args.contains(&"--dangerously-skip-permissions".to_string()));
    }

    #[test]
    fn agy_prompt_with_special_chars_is_a_single_argument_after_dash_p() {
        let args = one_shot_args(CliKind::Antigravity, SPECIAL_PROMPT_FIXTURE);
        assert_eq!(args[0], "--sandbox");
        assert_eq!(args[1], "-p");
        assert_eq!(args[2], SPECIAL_PROMPT_FIXTURE);
        assert!(args.contains(&"--sandbox".to_string()));
        assert!(args_are_safe(&args));
        assert!(!args.contains(&"--dangerously-skip-permissions".to_string()));
        // The prompt itself may contain the text -p, but it is not a flag.
        assert_eq!(args.iter().filter(|arg| *arg == "-p").count(), 1);
    }

    #[test]
    fn rejects_cmd_and_bat_when_no_real_exe_exists() {
        let dir = tempdir().unwrap();
        let cmd = dir.path().join("opencode.cmd");
        let bat = dir.path().join("agy.bat");
        fs::write(&cmd, b"@echo off\n").unwrap();
        fs::write(&bat, b"@echo off\n").unwrap();
        assert_eq!(
            resolve_real_executable(CliKind::OpenCode, &cmd).unwrap_err(),
            CliError::ScriptWrapper
        );
        assert_eq!(
            resolve_real_executable(CliKind::Antigravity, &bat).unwrap_err(),
            CliError::ScriptWrapper
        );
        assert!(find_in_directories(CliKind::OpenCode, [dir.path().to_path_buf()]).is_none());
    }

    #[test]
    fn resolves_sibling_exe_instead_of_running_cmd() {
        let dir = tempdir().unwrap();
        let cmd = dir.path().join("opencode.cmd");
        let exe = dir.path().join("opencode.exe");
        fs::write(&cmd, b"@echo off\n").unwrap();
        fs::write(&exe, b"not-executed").unwrap();
        let resolved = resolve_real_executable(CliKind::OpenCode, &cmd).unwrap();
        assert_eq!(resolved, exe);
        assert!(!is_script_wrapper(&resolved));
        let found = find_in_directories(CliKind::OpenCode, [dir.path().to_path_buf()]).unwrap();
        assert_eq!(found, exe);
    }

    #[test]
    fn workdir_is_empty_and_does_not_read_home_credential_paths() {
        let dir = tempdir().unwrap();
        let home = dir.path().join("fake-home");
        fs::create_dir_all(home.join(".opencode")).unwrap();
        fs::write(
            home.join(".opencode").join("auth.json"),
            b"{\"secret\":\"nope\"}",
        )
        .unwrap();
        fs::create_dir_all(home.join(".gemini").join("antigravity-cli")).unwrap();
        fs::write(
            home.join(".gemini")
                .join("antigravity-cli")
                .join("credentials.json"),
            b"{\"secret\":\"nope\"}",
        )
        .unwrap();
        let root = dir.path().join("work");
        let prepared = prepare_workdir(&root, CliKind::OpenCode).unwrap();
        let config = fs::read_to_string(prepared.join("opencode.json")).unwrap();
        assert!(config.contains("\"effect\": \"deny\""));
        // We never open the fake home credential files as part of prepare.
        assert!(!prepared.starts_with(&home));
        let entries: Vec<_> = fs::read_dir(&prepared)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn spawn_without_shell_keeps_the_special_prompt_intact() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("echo_args.rs");
        fs::write(
            &src,
            r#"
            fn main() {
                let args: Vec<String> = std::env::args().skip(1).collect();
                print!("{}", args.join("\u{1f}"));
            }
            "#,
        )
        .unwrap();
        let exe = dir
            .path()
            .join(format!("agy{}", std::env::consts::EXE_SUFFIX));
        let rustc = std::env::var_os("CARGO")
            .map(PathBuf::from)
            .and_then(|p| p.parent().map(|parent| parent.join("rustc")));
        let rustc = rustc
            .filter(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from("rustc"));
        let status = Command::new(rustc)
            .arg(&src)
            .arg("-o")
            .arg(&exe)
            .status()
            .expect("rustc");
        assert!(status.success(), "failed to compile argv echo fixture");
        let args = one_shot_args(CliKind::Antigravity, SPECIAL_PROMPT_FIXTURE);
        let output = Command::new(&exe)
            .args(&args)
            .output()
            .expect("spawn fake agy without a shell");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        let received: Vec<&str> = stdout.split('\u{1f}').collect();
        assert_eq!(received[0], "--sandbox");
        assert_eq!(received[1], "-p");
        assert_eq!(received[2], SPECIAL_PROMPT_FIXTURE);
    }
}
