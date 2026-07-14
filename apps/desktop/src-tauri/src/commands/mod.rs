use crate::domain::{AppPreferences, CommandError, ErrorKind, OneShotRequest, OneShotResult, ProviderStatus, SavedPosition};
use crate::services::monitor_position::restore_mascot_position;
use crate::AppState;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

fn internal_error(message: impl Into<String>) -> CommandError {
    CommandError::new(ErrorKind::Unknown, message)
}

#[tauri::command]
pub async fn detect_codex(state: State<'_, AppState>) -> Result<ProviderStatus, CommandError> {
    let preferences = state.preferences.get();
    Ok(state.codex.detect(preferences.codex_path.as_deref()).await)
}

#[tauri::command]
pub async fn ask_codex(
    state: State<'_, AppState>,
    request: OneShotRequest,
) -> Result<OneShotResult, CommandError> {
    let preferences = state.preferences.get();
    state
        .codex
        .ask(request, preferences.codex_path.as_deref())
        .await
}

#[tauri::command]
pub async fn cancel_codex(
    state: State<'_, AppState>,
    request_id: String,
) -> Result<(), CommandError> {
    state.codex.cancel(&request_id).await
}

#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> AppPreferences {
    state.preferences.get()
}

#[tauri::command]
pub fn update_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    preferences: AppPreferences,
) -> Result<AppPreferences, CommandError> {
    let preferences = preferences.validate();
    let autostart = app.autolaunch();
    if preferences.launch_at_startup {
        autostart.enable().map_err(|error| internal_error(error.to_string()))?;
    } else {
        autostart.disable().map_err(|error| internal_error(error.to_string()))?;
    }
    let saved = state
        .preferences
        .replace(preferences)
        .map_err(internal_error)?;
    if let Some(window) = app.get_webview_window("mascot") {
        window
            .set_always_on_top(saved.always_on_top)
            .map_err(|error| internal_error(error.to_string()))?;
        window
            .set_ignore_cursor_events(saved.click_through)
            .map_err(|error| internal_error(error.to_string()))?;
    }
    Ok(saved)
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), CommandError> {
    open_settings_internal(&app)
}

#[tauri::command]
pub fn open_quick_ask(app: AppHandle, state: State<'_, AppState>) -> Result<(), CommandError> {
    show_quick_ask_internal(&app, &state)
}

#[tauri::command]
pub fn set_click_through(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<AppPreferences, CommandError> {
    set_click_through_internal(&app, &state, enabled)
}

#[tauri::command]
pub fn set_always_on_top(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<AppPreferences, CommandError> {
    set_always_on_top_internal(&app, &state, enabled)
}

#[tauri::command]
pub fn save_mascot_position(
    state: State<'_, AppState>,
    position: SavedPosition,
) -> Result<AppPreferences, CommandError> {
    let mut preferences = state.preferences.get();
    preferences.mascot_position = Some(position);
    state
        .preferences
        .replace(preferences)
        .map_err(internal_error)
}

#[tauri::command]
pub fn reset_mascot_position(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppPreferences, CommandError> {
    let mut preferences = state.preferences.get();
    preferences.mascot_position = None;
    let preferences = state
        .preferences
        .replace(preferences)
        .map_err(internal_error)?;
    restore_mascot_position(&app, None).map_err(internal_error)?;
    Ok(preferences)
}

#[tauri::command]
pub async fn quit_app(app: AppHandle, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.codex.cancel_all().await;
    state.logger.event(crate::services::logging::SafeEvent::AppStopped);
    app.exit(0);
    Ok(())
}

pub fn open_settings_internal(app: &AppHandle) -> Result<(), CommandError> {
    if let Some(mascot) = app.get_webview_window("mascot") {
        mascot
            .hide()
            .map_err(|error| internal_error(error.to_string()))?;
    }
    let window = app
        .get_webview_window("settings")
        .ok_or_else(|| internal_error("Settings window is unavailable."))?;
    window.show().map_err(|error| internal_error(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| internal_error(error.to_string()))
}

pub fn show_quick_ask_internal(app: &AppHandle, state: &AppState) -> Result<(), CommandError> {
    let _ = set_click_through_internal(app, state, false)?;
    let window = app
        .get_webview_window("mascot")
        .ok_or_else(|| internal_error("Mascot window is unavailable."))?;
    window.show().map_err(|error| internal_error(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| internal_error(error.to_string()))?;
    window
        .emit("quick-ask:open", ())
        .map_err(|error| internal_error(error.to_string()))
}

pub fn set_click_through_internal(
    app: &AppHandle,
    state: &AppState,
    enabled: bool,
) -> Result<AppPreferences, CommandError> {
    let window = app
        .get_webview_window("mascot")
        .ok_or_else(|| internal_error("Mascot window is unavailable."))?;
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|error| internal_error(error.to_string()))?;
    let mut preferences = state.preferences.get();
    preferences.click_through = enabled;
    state
        .preferences
        .replace(preferences)
        .map_err(internal_error)
}

pub fn set_always_on_top_internal(
    app: &AppHandle,
    state: &AppState,
    enabled: bool,
) -> Result<AppPreferences, CommandError> {
    let window = app
        .get_webview_window("mascot")
        .ok_or_else(|| internal_error("Mascot window is unavailable."))?;
    window
        .set_always_on_top(enabled)
        .map_err(|error| internal_error(error.to_string()))?;
    let mut preferences = state.preferences.get();
    preferences.always_on_top = enabled;
    state
        .preferences
        .replace(preferences)
        .map_err(internal_error)
}
