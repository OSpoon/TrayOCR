use tauri::image::Image;
use tauri::menu::{
    AboutMetadata, CheckMenuItemBuilder, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use tauri::{App, AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_opener::OpenerExt;

const GITHUB_REPO_URL: &str = "https://github.com/OSpoon/tray-ocr-app";
const ISSUES_URL: &str = "https://github.com/OSpoon/tray-ocr-app/issues";

pub fn setup_menu<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    // macOS App menu (About / Theme / Check for Updates / Services / Hide / Hide Others / Show All / Quit)
    let about_icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;
    let about = PredefinedMenuItem::about(
        app,
        Some("About TrayOCR"),
        Some(AboutMetadata {
            icon: Some(about_icon),
            copyright: Some("© 2026 OSpoon".to_string()),
            ..Default::default()
        }),
    )?;

    let services = PredefinedMenuItem::services(app, None)?;
    let hide = PredefinedMenuItem::hide(app, None)?;
    let hide_others = PredefinedMenuItem::hide_others(app, None)?;
    let show_all = PredefinedMenuItem::show_all(app, None)?;
    let quit = PredefinedMenuItem::quit(app, None)?;
    let sep = PredefinedMenuItem::separator(app)?;

    // Custom items with initial check status
    let app_data_dir = app.path().app_data_dir().ok();
    let dark_mode = app_data_dir
        .as_ref()
        .map(|d| crate::config::load(d).dark_mode)
        .unwrap_or(false);

    let theme_light = CheckMenuItemBuilder::with_id("theme_light", "Light")
        .checked(!dark_mode)
        .build(app)?;
    let theme_dark = CheckMenuItemBuilder::with_id("theme_dark", "Dark")
        .checked(dark_mode)
        .build(app)?;
    let theme_system = CheckMenuItemBuilder::with_id("theme_system", "System")
        .checked(false)
        .build(app)?;

    let theme_menu = Submenu::with_items(
        app,
        "Theme",
        true,
        &[&theme_light, &theme_dark, &theme_system],
    )?;
    let check_updates = MenuItem::with_id(
        app,
        "check_updates",
        "Check for Updates…",
        true,
        None::<&str>,
    )?;

    let app_menu = Submenu::with_items(
        app,
        app.package_info().name.clone(),
        true,
        &[
            &about,
            &sep,
            &theme_menu,
            &check_updates,
            &sep,
            &services,
            &sep,
            &hide,
            &hide_others,
            &show_all,
            &sep,
            &quit,
        ],
    )?;

    // Help
    let help_github = MenuItem::with_id(app, "help_github", "GitHub", true, None::<&str>)?;
    let help_issues = MenuItem::with_id(app, "help_issues", "Issues", true, None::<&str>)?;
    let help_menu = Submenu::with_items(app, "Help", true, &[&help_github, &help_issues])?;

    let menu = Menu::with_items(app, &[&app_menu, &help_menu])?;

    app.set_menu(menu)?;
    Ok(())
}

fn find_check_item<R: Runtime>(menu: &Menu<R>, id: &str) -> Option<tauri::menu::CheckMenuItem<R>> {
    if let Ok(items) = menu.items() {
        for item in items {
            if let Some(found) = find_check_item_in_kind(item, id) {
                return Some(found);
            }
        }
    }
    None
}

fn find_check_item_in_kind<R: Runtime>(
    item: tauri::menu::MenuItemKind<R>,
    id: &str,
) -> Option<tauri::menu::CheckMenuItem<R>> {
    match item {
        tauri::menu::MenuItemKind::Check(check_item) => {
            if check_item.id().as_ref() == id {
                Some(check_item)
            } else {
                None
            }
        }
        tauri::menu::MenuItemKind::Submenu(submenu) => {
            if let Ok(sub_items) = submenu.items() {
                for sub_item in sub_items {
                    if let Some(found) = find_check_item_in_kind(sub_item, id) {
                        return Some(found);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    match event.id().as_ref() {
        "check_updates" => {
            let _ = app.emit("check-update", ());
        }
        "theme_light" => {
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                let mut cfg = crate::config::load(&app_data_dir);
                cfg.dark_mode = false;
                crate::config::save(&app_data_dir, &cfg);
            }
            if let Some(m) = app.menu() {
                if let Some(item) = find_check_item(&m, "theme_light") {
                    let _ = item.set_checked(true);
                }
                if let Some(item) = find_check_item(&m, "theme_dark") {
                    let _ = item.set_checked(false);
                }
                if let Some(item) = find_check_item(&m, "theme_system") {
                    let _ = item.set_checked(false);
                }
            }
            let _ = app.emit("set-theme", "light");
        }
        "theme_dark" => {
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                let mut cfg = crate::config::load(&app_data_dir);
                cfg.dark_mode = true;
                crate::config::save(&app_data_dir, &cfg);
            }
            if let Some(m) = app.menu() {
                if let Some(item) = find_check_item(&m, "theme_light") {
                    let _ = item.set_checked(false);
                }
                if let Some(item) = find_check_item(&m, "theme_dark") {
                    let _ = item.set_checked(true);
                }
                if let Some(item) = find_check_item(&m, "theme_system") {
                    let _ = item.set_checked(false);
                }
            }
            let _ = app.emit("set-theme", "dark");
        }
        "theme_system" => {
            if let Some(m) = app.menu() {
                if let Some(item) = find_check_item(&m, "theme_light") {
                    let _ = item.set_checked(false);
                }
                if let Some(item) = find_check_item(&m, "theme_dark") {
                    let _ = item.set_checked(false);
                }
                if let Some(item) = find_check_item(&m, "theme_system") {
                    let _ = item.set_checked(true);
                }
            }
            let _ = app.emit("set-theme", "auto");
        }
        "help_github" => {
            let _ = app.opener().open_url(GITHUB_REPO_URL, None::<&str>);
        }
        "help_issues" => {
            let _ = app.opener().open_url(ISSUES_URL, None::<&str>);
        }
        _ => {}
    }
}
