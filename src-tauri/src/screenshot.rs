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

#[cfg(target_os = "windows")]
fn capture_screenshot(path: &std::path::Path) -> std::io::Result<std::process::ExitStatus> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    const SCRIPT: &str = r#"
param([Parameter(Mandatory = $true)][string]$OutputPath)
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
$form = New-Object System.Windows.Forms.Form
$form.FormBorderStyle = [System.Windows.Forms.FormBorderStyle]::None
$form.StartPosition = [System.Windows.Forms.FormStartPosition]::Manual
$form.Bounds = $bounds
$form.TopMost = $true
$form.BackColor = [System.Drawing.Color]::Black
$form.Opacity = 0.18
$form.Cursor = [System.Windows.Forms.Cursors]::Cross
$form.KeyPreview = $true

$state = @{
  Dragging = $false
  Start = [System.Drawing.Point]::Empty
  Current = [System.Drawing.Point]::Empty
}

$getRect = {
  $x = [Math]::Min($state.Start.X, $state.Current.X)
  $y = [Math]::Min($state.Start.Y, $state.Current.Y)
  $w = [Math]::Abs($state.Start.X - $state.Current.X)
  $h = [Math]::Abs($state.Start.Y - $state.Current.Y)
  New-Object System.Drawing.Rectangle -ArgumentList $x, $y, $w, $h
}

$form.Add_MouseDown({
  $state.Dragging = $true
  $state.Start = $_.Location
  $state.Current = $_.Location
})

$form.Add_MouseMove({
  if ($state.Dragging) {
    $state.Current = $_.Location
    $form.Invalidate()
  }
})

$form.Add_MouseUp({
  if (-not $state.Dragging) { return }
  $state.Dragging = $false
  $state.Current = $_.Location
  $rect = & $getRect
  if ($rect.Width -lt 3 -or $rect.Height -lt 3) {
    $form.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
    $form.Close()
    return
  }

  $screenRect = New-Object System.Drawing.Rectangle -ArgumentList @(
    ($bounds.X + $rect.X),
    ($bounds.Y + $rect.Y),
    $rect.Width,
    $rect.Height
  )
  $form.Hide()
  Start-Sleep -Milliseconds 150

  $bitmap = New-Object System.Drawing.Bitmap -ArgumentList $screenRect.Width, $screenRect.Height
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.CopyFromScreen($screenRect.Location, [System.Drawing.Point]::Empty, $screenRect.Size)
  $bitmap.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
  $graphics.Dispose()
  $bitmap.Dispose()

  $form.DialogResult = [System.Windows.Forms.DialogResult]::OK
  $form.Close()
})

$form.Add_Paint({
  if (-not $state.Dragging) { return }
  $rect = & $getRect
  $pen = New-Object System.Drawing.Pen -ArgumentList ([System.Drawing.Color]::DodgerBlue), 2
  $_.Graphics.DrawRectangle($pen, $rect)
  $pen.Dispose()
})

$form.Add_KeyDown({
  if ($_.KeyCode -eq [System.Windows.Forms.Keys]::Escape) {
    $form.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
    $form.Close()
  }
})

$result = $form.ShowDialog()
if ($result -ne [System.Windows.Forms.DialogResult]::OK -or -not (Test-Path $OutputPath)) {
  exit 1
}
"#;

    let script_path = temp_path("tray-ocr-capture", "ps1");
    std::fs::write(&script_path, SCRIPT)?;
    let status = std::process::Command::new("powershell.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .arg("-NoProfile")
        .arg("-WindowStyle")
        .arg("Hidden")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script_path)
        .arg(path)
        .status();
    let _ = std::fs::remove_file(script_path);
    status
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn capture_screenshot(_path: &std::path::Path) -> std::io::Result<std::process::ExitStatus> {
    std::process::Command::new("false").status()
}
