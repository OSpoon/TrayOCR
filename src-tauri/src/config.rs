use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri_plugin_global_shortcut::{Code, Modifiers};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl ShortcutConfig {
    pub fn default_shortcut() -> Self {
        Self {
            modifiers: vec!["Alt".into()],
            key: "KeyA".into(),
        }
    }

    pub fn to_modifiers(&self) -> Option<Modifiers> {
        let mut m = Modifiers::empty();
        for mod_str in &self.modifiers {
            match mod_str.as_str() {
                "Alt" => m |= Modifiers::ALT,
                "Ctrl" | "Control" => m |= Modifiers::CONTROL,
                "Shift" => m |= Modifiers::SHIFT,
                "Super" | "Command" | "Meta" => m |= Modifiers::SUPER,
                _ => {}
            }
        }
        if m.is_empty() {
            None
        } else {
            Some(m)
        }
    }

    pub fn to_key(&self) -> Option<Code> {
        Some(match self.key.as_str() {
            "KeyA" => Code::KeyA,
            "KeyB" => Code::KeyB,
            "KeyC" => Code::KeyC,
            "KeyD" => Code::KeyD,
            "KeyE" => Code::KeyE,
            "KeyF" => Code::KeyF,
            "KeyG" => Code::KeyG,
            "KeyH" => Code::KeyH,
            "KeyI" => Code::KeyI,
            "KeyJ" => Code::KeyJ,
            "KeyK" => Code::KeyK,
            "KeyL" => Code::KeyL,
            "KeyM" => Code::KeyM,
            "KeyN" => Code::KeyN,
            "KeyO" => Code::KeyO,
            "KeyP" => Code::KeyP,
            "KeyQ" => Code::KeyQ,
            "KeyR" => Code::KeyR,
            "KeyS" => Code::KeyS,
            "KeyT" => Code::KeyT,
            "KeyU" => Code::KeyU,
            "KeyV" => Code::KeyV,
            "KeyW" => Code::KeyW,
            "KeyX" => Code::KeyX,
            "KeyY" => Code::KeyY,
            "KeyZ" => Code::KeyZ,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcut: ShortcutConfig,
    pub dark_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: ShortcutConfig::default_shortcut(),
            dark_mode: false,
        }
    }
}

fn config_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("config.json")
}

pub fn load(app_data_dir: &Path) -> AppConfig {
    let path = config_path(app_data_dir);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app_data_dir: &Path, config: &AppConfig) {
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(config_path(app_data_dir), content);
    }
}
