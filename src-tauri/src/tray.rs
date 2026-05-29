use tauri::{
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt;

pub fn build_menu(app: &tauri::AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, tauri::Error> {
    let screenshot = MenuItemBuilder::with_id("screenshot", "Screenshot OCR").build(app)?;
    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;

    let is_pinned = app
        .get_webview_window("main")
        .and_then(|w| w.is_always_on_top().ok())
        .unwrap_or(false);
    let pin = CheckMenuItemBuilder::with_id("pin", "Always on Top")
        .checked(is_pinned)
        .build(app)?;

    let shortcut = MenuItemBuilder::with_id("shortcut", "Change Hotkey").build(app)?;

    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart = CheckMenuItemBuilder::with_id("autostart", "Launch at Login")
        .checked(autostart_enabled)
        .build(app)?;

    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    MenuBuilder::new(app)
        .item(&screenshot)
        .item(&show)
        .separator()
        .item(&pin)
        .item(&shortcut)
        .item(&autostart)
        .separator()
        .item(&quit)
        .build()
}

pub fn setup(
    app: &tauri::AppHandle,
    perform_screenshot: fn(tauri::AppHandle),
) -> Result<(), tauri::Error> {
    let menu = build_menu(app)?;

    let img_bytes = include_bytes!("../icons/tray-icon.png");
    let icon = tauri::image::Image::from_bytes(img_bytes).expect("failed to load tray icon");

    TrayIconBuilder::new()
        .tooltip("TrayOCR")
        .icon_as_template(false)
        .icon(icon)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "screenshot" => perform_screenshot(app.clone()),
            "pin" => {
                if let Some(win) = app.get_webview_window("main") {
                    let on_top = win.is_always_on_top().unwrap_or(false);
                    let _ = win.set_always_on_top(!on_top);
                }
                refresh_menu(app);
            }
            "show" | "shortcut" => {
                if let Some(win) = app.get_webview_window("main") {
                    if event.id.as_ref() == "shortcut" {
                        let _ = app.emit("record-shortcut", ());
                    }
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "autostart" => {
                let enabled = app.autolaunch().is_enabled().unwrap_or(false);
                if enabled {
                    let _ = app.autolaunch().disable();
                } else {
                    let _ = app.autolaunch().enable();
                }
                refresh_menu(app);
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                if let Ok(new_menu) = build_menu(app) {
                    let _ = tray.set_menu(Some(new_menu));
                }
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn refresh_menu(app: &tauri::AppHandle) {
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(new_menu) = build_menu(app) {
            let _ = tray.set_menu(Some(new_menu));
        }
    }
}
