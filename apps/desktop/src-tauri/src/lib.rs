mod commands;
mod domain;
mod services;

use crate::services::app_paths::AppPaths;
use crate::services::codex_process::CodexService;
use crate::services::logging::{SafeEvent, SafeLogger};
use crate::services::monitor_position::restore_mascot_position;
use crate::services::preferences::PreferencesStore;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, WindowEvent};

pub struct AppState {
    pub codex: CodexService,
    pub preferences: PreferencesStore,
    pub logger: SafeLogger,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let paths = AppPaths::discover().expect("Windows application paths");
    paths.ensure().expect("application directories");
    let logger = SafeLogger::new(paths.log_dir.clone());
    logger.event(SafeEvent::AppStarted);
    let preferences = PreferencesStore::load(paths.config_path.clone());
    let mut startup_preferences = preferences.get();
    if startup_preferences.click_through {
        startup_preferences.click_through = false;
        let _ = preferences.replace(startup_preferences);
    }
    let codex = CodexService::new(paths.neutral_work_dir, logger.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState {
            codex,
            preferences,
            logger,
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_codex,
            commands::ask_codex,
            commands::cancel_codex,
            commands::get_preferences,
            commands::update_preferences,
            commands::open_settings,
            commands::open_quick_ask,
            commands::set_click_through,
            commands::set_always_on_top,
            commands::save_mascot_position,
            commands::reset_mascot_position,
            commands::quit_app,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            let state = app.state::<AppState>();
            let preferences = state.preferences.get();
            if let Some(window) = app.get_webview_window("mascot") {
                window.set_always_on_top(preferences.always_on_top)?;
                window.set_ignore_cursor_events(false)?;
            }
            let _ = restore_mascot_position(&app_handle, preferences.mascot_position.as_ref());
            build_tray(app)?;

            for label in ["mascot", "settings"] {
                if let Some(window) = app.get_webview_window(label) {
                    let handle = app_handle.clone();
                    let owned_label = label.to_string();
                    window.on_window_event(move |event| {
                        if let WindowEvent::CloseRequested { api, .. } = event {
                            api.prevent_close();
                            if let Some(window) = handle.get_webview_window(&owned_label) {
                                let _ = window.hide();
                            }
                            if owned_label == "settings" {
                                if let Some(mascot) = handle.get_webview_window("mascot") {
                                    let _ = mascot.show();
                                }
                            }
                        }
                    });
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenFamiliar");
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let preferences = state.preferences.get();
    let ask = MenuItem::with_id(app, "ask", "Ask Codex", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let always_on_top = CheckMenuItem::with_id(
        app,
        "always_on_top",
        "Always on top",
        true,
        preferences.always_on_top,
        None::<&str>,
    )?;
    let click_through = CheckMenuItem::with_id(
        app,
        "click_through",
        "Click-through",
        true,
        false,
        None::<&str>,
    )?;
    let reset = MenuItem::with_id(app, "reset_position", "Reset position", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &ask,
            &settings,
            &separator,
            &always_on_top,
            &click_through,
            &reset,
            &about,
            &separator,
            &quit,
        ],
    )?;
    let always_item = always_on_top.clone();
    let click_item = click_through.clone();
    let click_item_for_tray = click_through.clone();
    let icon = app.default_window_icon().cloned();
    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("OpenFamiliar — Perrito Tech")
        .on_menu_event(move |app, event| {
            let state = app.state::<AppState>();
            match event.id.as_ref() {
                "ask" => {
                    if commands::show_quick_ask_internal(app, &state).is_ok() {
                        let _ = click_item.set_checked(false);
                    }
                }
                "settings" => {
                    let _ = commands::open_settings_internal(app);
                }
                "always_on_top" => {
                    let next = !state.preferences.get().always_on_top;
                    if commands::set_always_on_top_internal(app, &state, next).is_ok() {
                        let _ = always_item.set_checked(next);
                    }
                }
                "click_through" => {
                    let next = !state.preferences.get().click_through;
                    if commands::set_click_through_internal(app, &state, next).is_ok() {
                        let _ = click_item.set_checked(next);
                    }
                }
                "reset_position" => {
                    let mut preferences = state.preferences.get();
                    preferences.mascot_position = None;
                    let _ = state.preferences.replace(preferences);
                    let _ = restore_mascot_position(app, None);
                }
                "about" => {
                    let _ = commands::open_settings_internal(app);
                    if let Some(window) = app.get_webview_window("settings") {
                        let _ = window.emit("settings:section", "about");
                    }
                }
                "quit" => {
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = handle.state::<AppState>();
                        state.codex.cancel_all().await;
                        state.logger.event(SafeEvent::AppStopped);
                        handle.exit(0);
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let state = app.state::<AppState>();
                if commands::show_quick_ask_internal(app, &state).is_ok() {
                    let _ = click_item_for_tray.set_checked(false);
                }
            }
        });
    if let Some(icon) = icon {
        builder = builder.icon(icon);
    }
    builder.build(app)?;
    Ok(())
}
