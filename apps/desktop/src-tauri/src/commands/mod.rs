use crate::domain::{
    AppPreferences, CommandError, ErrorKind, OneShotRequest, OneShotResult, ProviderStatus,
    SavedPosition,
};
use crate::services::monitor_position::restore_mascot_position;
use crate::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, State};
use tauri_plugin_autostart::ManagerExt;

fn internal_error(message: impl Into<String>) -> CommandError {
    CommandError::new(ErrorKind::Unknown, message)
}

const MASCOT_FRAME_SIZE: f64 = 64.0;
const MASCOT_WINDOW_PADDING: f64 = 24.0;
const QUICK_ASK_WIDTH: f64 = 560.0;
const QUICK_ASK_HEIGHT: f64 = 360.0;

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
    sync_autostart(&app, preferences.launch_at_startup)?;
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
    set_mascot_expanded_internal(&app, &state, state.mascot_expanded.load(Ordering::SeqCst))?;
    Ok(saved)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutostartChange {
    Enable,
    Disable,
}

fn required_autostart_change(current: bool, desired: bool) -> Option<AutostartChange> {
    match (current, desired) {
        (false, true) => Some(AutostartChange::Enable),
        (true, false) => Some(AutostartChange::Disable),
        _ => None,
    }
}

fn sync_autostart(app: &AppHandle, desired: bool) -> Result<(), CommandError> {
    let autostart = app.autolaunch();
    let current = autostart.is_enabled().map_err(|error| {
        internal_error(format!(
            "Could not inspect the Windows startup registration: {error}"
        ))
    })?;
    let result = match required_autostart_change(current, desired) {
        Some(AutostartChange::Enable) => autostart.enable(),
        Some(AutostartChange::Disable) => autostart.disable(),
        None => return Ok(()),
    };
    result.map_err(|error| {
        internal_error(format!(
            "Could not update the Windows startup registration: {error}"
        ))
    })
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
pub fn set_mascot_expanded(
    app: AppHandle,
    state: State<'_, AppState>,
    expanded: bool,
) -> Result<(), CommandError> {
    set_mascot_expanded_internal(&app, &state, expanded)
}

#[tauri::command]
pub fn save_mascot_position(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppPreferences, CommandError> {
    let window = app
        .get_webview_window("mascot")
        .ok_or_else(|| internal_error("Mascot window is unavailable."))?;
    let mut preferences = state.preferences.get();
    let scale_factor = window
        .scale_factor()
        .map_err(|error| internal_error(error.to_string()))?;
    let position = window
        .outer_position()
        .map_err(|error| internal_error(error.to_string()))?;
    let current_size = window
        .outer_size()
        .map_err(|error| internal_error(error.to_string()))?;
    let compact_size =
        mascot_window_size(preferences.scale, false).to_physical::<u32>(scale_factor);
    let monitor = window
        .current_monitor()
        .map_err(|error| internal_error(error.to_string()))?;
    preferences.mascot_position = Some(SavedPosition {
        x: anchored_axis(position.x, current_size.width, compact_size.width),
        y: anchored_axis(position.y, current_size.height, compact_size.height),
        monitor_name: monitor.as_ref().and_then(|current| current.name().cloned()),
        scale_factor,
    });
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
    state
        .logger
        .event(crate::services::logging::SafeEvent::AppStopped);
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
    window
        .show()
        .map_err(|error| internal_error(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| internal_error(error.to_string()))
}

pub fn show_quick_ask_internal(app: &AppHandle, state: &AppState) -> Result<(), CommandError> {
    let _ = set_click_through_internal(app, state, false)?;
    set_mascot_expanded_internal(app, state, true)?;
    let window = app
        .get_webview_window("mascot")
        .ok_or_else(|| internal_error("Mascot window is unavailable."))?;
    window
        .show()
        .map_err(|error| internal_error(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| internal_error(error.to_string()))?;
    window
        .emit("quick-ask:open", ())
        .map_err(|error| internal_error(error.to_string()))
}

pub fn set_mascot_expanded_internal(
    app: &AppHandle,
    state: &AppState,
    expanded: bool,
) -> Result<(), CommandError> {
    let window = app
        .get_webview_window("mascot")
        .ok_or_else(|| internal_error("Mascot window is unavailable."))?;
    let scale_factor = window
        .scale_factor()
        .map_err(|error| internal_error(error.to_string()))?;
    let current_position = window
        .outer_position()
        .map_err(|error| internal_error(error.to_string()))?;
    let current_size = window
        .outer_size()
        .map_err(|error| internal_error(error.to_string()))?;
    let target_logical = mascot_window_size(state.preferences.get().scale, expanded);
    let target_physical = target_logical.to_physical::<u32>(scale_factor);
    let mut target_position = PhysicalPosition::new(
        anchored_axis(
            current_position.x,
            current_size.width,
            target_physical.width,
        ),
        anchored_axis(
            current_position.y,
            current_size.height,
            target_physical.height,
        ),
    );
    if let Some(monitor) = window
        .current_monitor()
        .map_err(|error| internal_error(error.to_string()))?
    {
        let work_area = monitor.work_area();
        target_position.x = clamp_axis_to_work_area(
            target_position.x,
            work_area.position.x,
            work_area.size.width,
            target_physical.width,
        );
        target_position.y = clamp_axis_to_work_area(
            target_position.y,
            work_area.position.y,
            work_area.size.height,
            target_physical.height,
        );
    }
    window
        .set_size(target_logical)
        .map_err(|error| internal_error(error.to_string()))?;
    window
        .set_position(target_position)
        .map_err(|error| internal_error(error.to_string()))?;
    state.mascot_expanded.store(expanded, Ordering::SeqCst);
    Ok(())
}

fn mascot_window_size(scale: u8, expanded: bool) -> LogicalSize<f64> {
    if expanded {
        return LogicalSize::new(QUICK_ASK_WIDTH, QUICK_ASK_HEIGHT);
    }
    let edge = MASCOT_FRAME_SIZE * f64::from(scale.clamp(1, 3)) + MASCOT_WINDOW_PADDING;
    LogicalSize::new(edge, edge)
}

fn anchored_axis(position: i32, current_size: u32, target_size: u32) -> i32 {
    let value = i64::from(position) + i64::from(current_size) - i64::from(target_size);
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn clamp_axis_to_work_area(value: i32, minimum: i32, work_size: u32, size: u32) -> i32 {
    let minimum = i64::from(minimum);
    let maximum = minimum + i64::from(work_size.saturating_sub(size));
    i64::from(value).clamp(minimum, maximum) as i32
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_compact_window_is_one_third_smaller_than_previous_sprite() {
        let size = mascot_window_size(2, false);
        assert_eq!(size, LogicalSize::new(152.0, 152.0));
    }

    #[test]
    fn resizing_preserves_bottom_right_anchor() {
        assert_eq!(anchored_axis(100, 560, 152), 508);
        assert_eq!(anchored_axis(508, 152, 560), 100);
    }

    #[test]
    fn expanded_window_is_clamped_to_negative_monitor_work_area() {
        assert_eq!(clamp_axis_to_work_area(-2_500, -1_920, 1_920, 560), -1_920);
        assert_eq!(clamp_axis_to_work_area(200, -1_920, 1_920, 560), -560);
    }

    #[test]
    fn autostart_change_is_idempotent() {
        assert_eq!(required_autostart_change(false, false), None);
        assert_eq!(required_autostart_change(true, true), None);
        assert_eq!(
            required_autostart_change(false, true),
            Some(AutostartChange::Enable)
        );
        assert_eq!(
            required_autostart_change(true, false),
            Some(AutostartChange::Disable)
        );
    }
}
