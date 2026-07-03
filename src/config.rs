use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    #[serde(default = "default_countdown")]
    pub countdown_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationConfig {
    #[serde(default = "default_notification_text")]
    pub text: String,
}

fn default_countdown() -> u64 {
    1200
}

fn default_notification_text() -> String {
    "\u{1F441}\u{FE0F} 该休息一下眼睛了！\nTime to rest your eyes!".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            countdown_seconds: default_countdown(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            text: default_notification_text(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            notification: NotificationConfig::default(),
        }
    }
}

fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("eye-friend")
        .join("config.toml")
}

pub fn load_or_create(path_arg: Option<PathBuf>) -> (Config, PathBuf) {
    let path = path_arg.unwrap_or_else(default_config_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if !path.exists() {
        let default = Config::default();
        if let Ok(s) = toml::to_string_pretty(&default) {
            let _ = fs::write(&path, s);
        }
        return (default, path);
    }
    match fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(cfg) => (cfg, path),
            Err(e) => {
                eprintln!("eye-friend: config parse error: {e}. Using defaults.");
                (Config::default(), path)
            }
        },
        Err(e) => {
            eprintln!("eye-friend: cannot read config: {e}. Using defaults.");
            (Config::default(), path)
        }
    }
}

pub fn reload(path: &PathBuf) -> Option<Config> {
    match fs::read_to_string(path) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                eprintln!("eye-friend: config reload parse error: {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("eye-friend: config reload read error: {e}");
            None
        }
    }
}
