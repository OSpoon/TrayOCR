#[cfg(not(target_os = "macos"))]
compile_error!("TrayOCR only supports macOS.");

mod config;
mod history;
mod menus;
mod ocr;
mod screenshot;
mod tray;

use std::path::PathBuf;

use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

struct AppState {
    app_data_dir: PathBuf,
}

#[tauri::command]
fn get_history(state: tauri::State<AppState>) -> Vec<history::HistoryEntry> {
    history::load(&state.app_data_dir)
}

#[tauri::command]
fn clear_history(state: tauri::State<AppState>) {
    history::clear(&state.app_data_dir);
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> config::AppConfig {
    config::load(&state.app_data_dir)
}

#[tauri::command]
fn get_shortcut(state: tauri::State<AppState>) -> config::ShortcutConfig {
    config::load(&state.app_data_dir).shortcut
}

#[tauri::command]
fn set_shortcut(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    modifiers: Vec<String>,
    key: String,
) -> Result<config::ShortcutConfig, String> {
    let old_cfg = config::load(&state.app_data_dir);

    let mut new_cfg = old_cfg.clone();
    new_cfg.shortcut = config::ShortcutConfig { modifiers, key };

    let mods = new_cfg.shortcut.to_modifiers();
    let code = new_cfg
        .shortcut
        .to_key()
        .ok_or_else(|| format!("Unsupported key: {}", new_cfg.shortcut.key))?;

    if let (Some(old_mods), Some(old_code)) =
        (old_cfg.shortcut.to_modifiers(), old_cfg.shortcut.to_key())
    {
        let _ = app
            .global_shortcut()
            .unregister(Shortcut::new(Some(old_mods), old_code));
    }

    app.global_shortcut()
        .register(Shortcut::new(mods, code))
        .map_err(|e| format!("Failed to register: {e}"))?;

    config::save(&state.app_data_dir, &new_cfg);
    Ok(new_cfg.shortcut)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_menu_event(|app, event| menus::handle_menu_event(app, event))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    let state = app.state::<AppState>();
                    let cfg = config::load(&state.app_data_dir).shortcut;
                    let expected_mods = cfg.to_modifiers();
                    let expected_code = cfg.to_key();
                    if event.state == ShortcutState::Pressed
                        && expected_mods.map(|m| shortcut.mods == m).unwrap_or(true)
                        && Some(shortcut.key) == expected_code
                    {
                        screenshot::perform(app.clone());
                    }
                })
                .build(),
        )
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir).ok();
            app.manage(AppState {
                app_data_dir: app_data_dir.clone(),
            });

            {
                let cfg = config::load(&app_data_dir);
                if let (Some(mods), Some(code)) =
                    (cfg.shortcut.to_modifiers(), cfg.shortcut.to_key())
                {
                    app.global_shortcut()
                        .register(Shortcut::new(Some(mods), code))?;
                }
            }

            menus::setup_menu(app)?;
            tray::setup(app.handle(), screenshot::perform)?;

            if let Some(win) = app.get_webview_window("main") {
                let win_ = win.clone();
                win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            clear_history,
            get_config,
            get_shortcut,
            set_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
