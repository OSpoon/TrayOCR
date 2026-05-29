use chrono::Local;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

use crate::history;

pub fn perform(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let screenshot_path = "/tmp/air-ocr-screenshot.png";

        let status = std::process::Command::new("screencapture")
            .args(["-i", screenshot_path])
            .status();

        match status {
            Ok(status) if status.success() => {
                match crate::ocr::recognize_file(screenshot_path) {
                    Ok(text) => {
                        if let Err(e) = app.clipboard().write_text(text.clone()) {
                            eprintln!("clipboard write failed: {}", e);
                        }
                        let app_data_dir = app.path().app_data_dir().expect("app data dir");
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
                            .title("Air OCR")
                            .body("Copied to clipboard")
                            .show();
                    }
                    Err(crate::ocr::OcrError::NoTextRecognized) => {
                        let _ = app
                            .notification()
                            .builder()
                            .title("Air OCR")
                            .body("No text found")
                            .show();
                    }
                    Err(e) => {
                        eprintln!("OCR error: {}", e);
                    }
                }
                let _ = std::fs::remove_file(screenshot_path);
            }
            Ok(_) => {
                let _ = std::fs::remove_file(screenshot_path);
            }
            Err(e) => {
                eprintln!("screencapture failed: {}", e);
            }
        }
    });
}
