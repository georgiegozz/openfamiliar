use familiar_core::{ChatMessageDto, FamiliarCore, MascotState};
use familiar_permissions::SecurityMode;
use familiar_storage::AppPaths;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State, WindowEvent,
};

pub struct AppState {
    pub core: Arc<FamiliarCore>,
}

#[derive(Serialize)]
struct CmdError {
    message: String,
}

impl From<String> for CmdError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

fn parse_mode(mode: &str) -> Result<SecurityMode, String> {
    match mode {
        "chat" => Ok(SecurityMode::Chat),
        "read_only" | "readonly" => Ok(SecurityMode::ReadOnly),
        "agent" => Ok(SecurityMode::Agent),
        other => Err(format!("unknown mode {other}")),
    }
}

#[tauri::command]
fn set_mascot_state(state: State<'_, AppState>, state_name: String) -> Result<(), CmdError> {
    let parsed = MascotState::parse(&state_name).ok_or_else(|| format!("bad state {state_name}"))?;
    state.core.set_state(parsed);
    Ok(())
}

// Frontend may pass { state } — support both via rename in invoke map using serde aliases in a wrapper
#[tauri::command]
fn set_mascot_state_v2(state: State<'_, AppState>, args: serde_json::Value) -> Result<(), CmdError> {
    let name = args
        .get("state")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "state required".to_string())?;
    set_mascot_state(state, name.to_string())
}

#[tauri::command]
async fn chat(
    state: State<'_, AppState>,
    provider_id: String,
    model: String,
    message: String,
    max_tokens: Option<u32>,
) -> Result<String, CmdError> {
    let text = state
        .core
        .chat_stream_collect(
            &provider_id,
            &model,
            vec![ChatMessageDto {
                role: "user".into(),
                content: message,
            }],
            max_tokens,
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(text)
}

#[tauri::command]
async fn chat_args(state: State<'_, AppState>, args: serde_json::Value) -> Result<String, CmdError> {
    let provider_id = args
        .get("providerId")
        .or_else(|| args.get("provider_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("mock")
        .to_string();
    let model = args
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("mock-model")
        .to_string();
    let message = args
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let max_tokens = args
        .get("maxTokens")
        .or_else(|| args.get("max_tokens"))
        .and_then(|v| v.as_u64())
        .map(|n| n as u32);
    chat(state, provider_id, model, message, max_tokens).await
}

#[tauri::command]
fn authorize_workspace(
    state: State<'_, AppState>,
    id: String,
    path: String,
) -> Result<(), CmdError> {
    state
        .core
        .authorize_workspace(&id, PathBuf::from(path))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn authorize_workspace_args(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> Result<(), CmdError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "path required".to_string())?
        .to_string();
    authorize_workspace(state, id, path)
}

#[tauri::command]
fn preview_workspace(
    state: State<'_, AppState>,
    id: String,
    paths: Vec<String>,
) -> Result<String, CmdError> {
    if paths.is_empty() {
        let tree = state.core.tree(&id, 40).map_err(|e| e.to_string())?;
        return Ok(format!("Tree (first 40):\n{}", tree.join("\n")));
    }
    let preview = state
        .core
        .preview_context(&id, &paths)
        .map_err(CmdError::from)?;
    Ok(serde_json::to_string_pretty(&preview).unwrap_or_default())
}

#[tauri::command]
fn preview_workspace_args(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> Result<String, CmdError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();
    let paths = args
        .get("paths")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    preview_workspace(state, id, paths)
}

#[tauri::command]
fn set_security_mode(state: State<'_, AppState>, mode: String) -> Result<(), CmdError> {
    let m = parse_mode(&mode)?;
    state.core.set_security_mode(m);
    Ok(())
}

#[tauri::command]
fn set_security_mode_args(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> Result<(), CmdError> {
    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "mode required".to_string())?;
    set_security_mode(state, mode.to_string())
}

#[tauri::command]
fn set_click_through(app: AppHandle, enabled: bool) -> Result<(), CmdError> {
    if let Some(win) = app.get_webview_window("main") {
        win.set_ignore_cursor_events(enabled)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn set_click_through_args(app: AppHandle, args: serde_json::Value) -> Result<(), CmdError> {
    let enabled = args
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    set_click_through(app, enabled)
}

#[tauri::command]
fn get_state(state: State<'_, AppState>) -> String {
    state.core.state().as_str().to_string()
}

#[tauri::command]
fn resize_window(app: AppHandle, width: f64, height: f64) -> Result<(), CmdError> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
    }
    Ok(())
}

fn load_env_file() {
    let mut log_content = String::new();
    let paths = vec![
        std::path::PathBuf::from(".env"),
        std::path::PathBuf::from("../.env"),
        std::path::PathBuf::from("../../.env"),
        std::path::PathBuf::from("../../../.env"),
    ];
    if let Ok(cd) = std::env::current_dir() {
        log_content.push_str(&format!("Checking env paths. Current dir: {:?}\n", cd));
    }
    for path in paths {
        log_content.push_str(&format!("Checking path: {:?}, exists: {}\n", path, path.exists()));
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut loaded_keys = Vec::new();
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, val)) = line.split_once('=') {
                        let key = key.trim();
                        let val = val.trim();
                        let val = val.strip_prefix('"').unwrap_or(val);
                        let val = val.strip_suffix('"').unwrap_or(val);
                        let val = val.strip_prefix('\'').unwrap_or(val);
                        let val = val.strip_suffix('\'').unwrap_or(val);
                        std::env::set_var(key, val);
                        loaded_keys.push(key.to_string());
                    }
                }
                log_content.push_str(&format!("Loaded keys from {:?}: {:?}\n", path, loaded_keys));
                break;
            }
        }
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("C:\\Users\\jorge gonzalez\\Music\\proyects\\OpenFamiliar\\tauri-diag.log")
    {
        use std::io::Write;
        let _ = writeln!(f, "{}", log_content);
    }
}

fn build_core() -> Result<FamiliarCore, String> {
    match AppPaths::discover() {
        Ok(paths) => {
            let _ = paths.ensure();
            FamiliarCore::open(&paths.db_path, &paths.audit_path).map_err(|e| e.to_string())
        }
        Err(_) => FamiliarCore::in_memory().map_err(|e| e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_env_file();
    let _ = std::fs::write("C:\\Users\\jorge gonzalez\\Music\\proyects\\OpenFamiliar\\tauri-diag.log", "run() started\n");
    let core = Arc::new(build_core().expect("core init"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState { core })
        .invoke_handler(tauri::generate_handler![
            set_mascot_state,
            set_mascot_state_v2,
            chat,
            chat_args,
            authorize_workspace,
            authorize_workspace_args,
            preview_workspace,
            preview_workspace_args,
            set_security_mode,
            set_security_mode_args,
            set_click_through,
            set_click_through_args,
            get_state,
            resize_window
        ])
        .setup(|app| {
            let has_main = app.get_webview_window("main").is_some();
            let windows: Vec<String> = app.webview_windows().keys().cloned().collect();
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("C:\\Users\\jorge gonzalez\\Music\\proyects\\OpenFamiliar\\tauri-diag.log")
            {
                use std::io::Write;
                let _ = writeln!(f, "setup() reached. has_main: {}, all windows: {:?}", has_main, windows);
            }
            // Map frontend camelCase commands to *_args handlers by registering aliases via JS.
            // Also create tray.
            let quit = MenuItem::with_id(app, "quit", "Quit OpenFamiliar", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("OpenFamiliar")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
                let app_handle = app.handle().clone();
                win.on_window_event(move |e| {
                    if let WindowEvent::CloseRequested { api, .. } = e {
                        // hide to tray instead of exit
                        api.prevent_close();
                        if let Some(w) = app_handle.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                });
            }
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("C:\\Users\\jorge gonzalez\\Music\\proyects\\OpenFamiliar\\tauri-diag.log")
                .map(|mut f| {
                    use std::io::Write;
                    let _ = writeln!(f, "setup() completed successfully");
                });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenFamiliar");
}
