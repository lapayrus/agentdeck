use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/generated/")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/generated/")]
pub struct WindowState {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: 1280,
            height: 820,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/generated/")]
pub struct AppSettings {
    pub theme: Theme,
    pub check_for_updates: bool,
    pub recent_projects: Vec<String>,
    pub binary_overrides: std::collections::HashMap<String, String>,
    pub window: WindowState,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            check_for_updates: true,
            recent_projects: Vec::new(),
            binary_overrides: std::collections::HashMap::new(),
            window: WindowState::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Store {
    directory: PathBuf,
}

impl Store {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    fn state_path(&self) -> PathBuf {
        self.directory.join("state.json")
    }

    pub fn load(&self) -> AppSettings {
        match fs::read_to_string(self.state_path()) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => AppSettings::default(),
        }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), String> {
        fs::create_dir_all(&self.directory)
            .map_err(|error| format!("Could not create the app data folder: {error}"))?;
        let text = serde_json::to_string_pretty(settings)
            .map_err(|error| format!("Could not serialize settings: {error}"))?;
        write_atomic(&self.state_path(), &text)
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

pub fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "The target has no parent directory.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    let stamp = std::process::id();
    let temporary = parent.join(format!(".agentdeck-write-{stamp}.tmp"));
    fs::write(&temporary, contents)
        .map_err(|error| format!("Could not write {}: {error}", temporary.display()))?;

    let mut last_error = None;
    for attempt in 0..6 {
        match fs::rename(&temporary, path) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                sleep(Duration::from_millis(40 * (attempt + 1)));
            }
        }
    }
    let _ = fs::remove_file(&temporary);
    Err(format!(
        "Could not replace {}: {}",
        path.display(),
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "unknown error".into())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn settings_round_trip() {
        let dir = tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        let mut settings = AppSettings::default();
        settings.theme = Theme::Dark;
        settings.recent_projects = vec!["C:\\repo".into()];
        store.save(&settings).unwrap();

        let loaded = store.load();
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.recent_projects, vec!["C:\\repo".to_string()]);
    }

    #[test]
    fn corrupt_state_falls_back_to_defaults() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("state.json"), "{not json").unwrap();
        let store = Store::new(dir.path().to_path_buf());
        assert_eq!(store.load().theme, Theme::System);
    }
}
