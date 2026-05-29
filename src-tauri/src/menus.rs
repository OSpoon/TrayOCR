use tauri::image::Image;
use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{App, AppHandle, Runtime};
use tauri_plugin_opener::OpenerExt;
use tauri::Emitter;

const GITHUB_REPO_URL: &str = "https://github.com/OSpoon/starter-tauri-app";
const ISSUES_URL: &str = "https://github.com/OSpoon/starter-tauri-app/issues";

pub fn setup_menu<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
  // macOS App menu (About / Theme / Check for Updates / Services / Hide / Hide Others / Show All / Quit)
  let about_icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;
  let about = PredefinedMenuItem::about(
    app,
    None,
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

  // Custom items
  let theme_light = MenuItem::with_id(app, "theme_light", "Light", true, None::<&str>)?;
  let theme_dark = MenuItem::with_id(app, "theme_dark", "Dark", true, None::<&str>)?;
  let theme_system = MenuItem::with_id(app, "theme_system", "System", true, None::<&str>)?;
  let theme_menu = Submenu::with_items(app, "Theme", true, &[&theme_light, &theme_dark, &theme_system])?;
  let check_updates = MenuItem::with_id(app, "check_updates", "Check for Updates…", true, None::<&str>)?;

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

  // Default menus (File / Edit / View / Window)
  let close_window = PredefinedMenuItem::close_window(app, None)?;
  let minimize = PredefinedMenuItem::minimize(app, None)?;
  let maximize = PredefinedMenuItem::maximize(app, None)?;
  let fullscreen = PredefinedMenuItem::fullscreen(app, None)?;

  let undo = PredefinedMenuItem::undo(app, None)?;
  let redo = PredefinedMenuItem::redo(app, None)?;
  let cut = PredefinedMenuItem::cut(app, None)?;
  let copy = PredefinedMenuItem::copy(app, None)?;
  let paste = PredefinedMenuItem::paste(app, None)?;
  let select_all = PredefinedMenuItem::select_all(app, None)?;

  let file_menu = Submenu::with_items(app, "File", true, &[&close_window])?;
  let edit_menu = Submenu::with_items(
    app,
    "Edit",
    true,
    &[&undo, &redo, &sep, &cut, &copy, &paste, &sep, &select_all],
  )?;
  let view_menu = Submenu::with_items(app, "View", true, &[&fullscreen])?;
  let window_menu = Submenu::with_items(app, "Window", true, &[&minimize, &maximize])?;

  let menu = Menu::with_items(
    app,
    &[&app_menu, &file_menu, &edit_menu, &view_menu, &window_menu, &help_menu],
  )?;

  app.set_menu(menu)?;
  Ok(())
}

pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
  match event.id().as_ref() {
    "check_updates" => {
      let _ = app.emit("app://check-updates", ());
    }
    "theme_light" => {
      let _ = app.emit("app://set-theme", "light");
    }
    "theme_dark" => {
      let _ = app.emit("app://set-theme", "dark");
    }
    "theme_system" => {
      let _ = app.emit("app://set-theme", "auto");
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

