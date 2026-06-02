use chrono::Local;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

use crate::history;

pub fn perform(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let screenshot_path = screenshot_path();

        let status = capture_screenshot(&screenshot_path);

        match status {
            Ok(status) if status.success() => {
                let app_data_dir = app.path().app_data_dir().expect("app data dir");
                match crate::ocr::recognize_file(&screenshot_path.to_string_lossy()) {
                    Ok(text) => {
                        if let Err(e) = app.clipboard().write_text(text.clone()) {
                            eprintln!("clipboard write failed: {}", e);
                        }
                        history::append(
                            &app_data_dir,
                            history::HistoryEntry {
                                id: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64,
                                text: text.clone(),
                                timestamp: Local::now().format("%H:%M").to_string(),
                            },
                        );

                        let _ = app.run_on_main_thread({
                            let app = app.clone();
                            move || {
                                if let Some(tray) = app.tray_by_id("main") {
                                    if let Ok(new_menu) = crate::tray::build_menu(&app) {
                                        let _ = tray.set_menu(Some(new_menu));
                                    }
                                }
                            }
                        });

                        let _ = app
                            .notification()
                            .builder()
                            .title("TrayOCR")
                            .body("Copied to clipboard")
                            .show();
                    }
                    Err(crate::ocr::OcrError::NoTextRecognized) => {
                        let _ = app
                            .notification()
                            .builder()
                            .title("TrayOCR")
                            .body("No text found")
                            .show();
                    }
                    Err(e) => {
                        eprintln!("OCR error: {}", e);
                        notify(&app, "TrayOCR", &format!("OCR failed: {}", e));
                    }
                }
                let _ = std::fs::remove_file(&screenshot_path);
            }
            Ok(_) => {
                let _ = std::fs::remove_file(&screenshot_path);
                notify(&app, "TrayOCR", "Screenshot cancelled or failed");
            }
            Err(e) => {
                eprintln!("screenshot failed: {}", e);
                notify(&app, "TrayOCR", &format!("Screenshot failed: {}", e));
            }
        }
    });
}

fn notify(app: &tauri::AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

fn screenshot_path() -> std::path::PathBuf {
    temp_path("tray-ocr-screenshot", "png")
}

fn temp_path(prefix: &str, extension: &str) -> std::path::PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{timestamp}.{extension}",
        std::process::id(),
    ))
}

#[cfg(target_os = "macos")]
fn capture_screenshot(path: &std::path::Path) -> std::io::Result<std::process::ExitStatus> {
    std::process::Command::new("screencapture")
        .arg("-i")
        .arg(path)
        .status()
}

#[cfg(not(target_os = "macos"))]
fn capture_screenshot(_path: &std::path::Path) -> std::io::Result<std::process::ExitStatus> {
    std::process::Command::new("false").status()
}
