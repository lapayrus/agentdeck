use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use agentdeck_core::model::{Ecosystem, ScanRequest, ScanResult};
use agentdeck_core::roots::{resolve_project_root, Roots};
use agentdeck_core::state::{AppSettings, Store};
use agentdeck_core::Manager;
use tauri::State;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub project_root: Mutex<Option<PathBuf>>,
    pub store: Store,
}

impl AppState {
    pub fn load() -> Self {
        let app_data = Roots::discover(None).app_data;
        let store = Store::new(app_data);
        let settings = store.load();
        Self {
            settings: Mutex::new(settings),
            project_root: Mutex::new(None),
            store,
        }
    }

    fn settings_snapshot(&self) -> AppSettings {
        self.settings
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    fn project_snapshot(&self) -> Option<PathBuf> {
        self.project_root
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

fn parse_ecosystem(key: &str) -> Option<Ecosystem> {
    match key {
        "claude" => Some(Ecosystem::Claude),
        "codex" => Some(Ecosystem::Codex),
        "opencode" => Some(Ecosystem::Opencode),
        "commandcode" => Some(Ecosystem::CommandCode),
        _ => None,
    }
}

fn build_manager(state: &AppState, project: Option<PathBuf>) -> Manager {
    let settings = state.settings_snapshot();
    let mut roots = Roots::discover(project);
    for (key, value) in &settings.binary_overrides {
        if let Some(ecosystem) = parse_ecosystem(key) {
            roots.overrides.insert(ecosystem, PathBuf::from(value));
        }
    }
    Manager::new(roots)
}

fn scan_with(state: &AppState, project: Option<PathBuf>) -> ScanResult {
    build_manager(state, project).scan()
}

#[tauri::command]
pub fn scan(request: Option<ScanRequest>, state: State<AppState>) -> Result<ScanResult, String> {
    let project = request
        .and_then(|value| value.project_root)
        .map(PathBuf::from)
        .or_else(|| state.project_snapshot());
    Ok(scan_with(&state, project))
}

#[tauri::command]
pub fn set_project(
    project_root: Option<String>,
    state: State<AppState>,
) -> Result<ScanResult, String> {
    let resolved = match project_root {
        Some(value) if !value.trim().is_empty() => {
            let path = PathBuf::from(&value);
            if !path.is_dir() {
                return Err(format!("{value} is not a folder."));
            }
            let (root, _) = resolve_project_root(&path);
            root
        }
        _ => {
            *state
                .project_root
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
            return Ok(scan_with(&state, None));
        }
    };

    *state
        .project_root
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(resolved.clone());

    let path_text = resolved.to_string_lossy().to_string();
    {
        let mut settings = state
            .settings
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        settings.recent_projects.retain(|entry| entry != &path_text);
        settings.recent_projects.insert(0, path_text);
        settings.recent_projects.truncate(12);
        if let Err(error) = state.store.save(&settings) {
            return Err(error);
        }
    }

    Ok(scan_with(&state, Some(resolved)))
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppSettings {
    state.settings_snapshot()
}

#[tauri::command]
pub fn save_settings(
    settings: AppSettings,
    state: State<AppState>,
) -> Result<AppSettings, String> {
    state.store.save(&settings)?;
    *state
        .settings
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = settings.clone();
    Ok(settings)
}

const MAX_RAW_FILE_BYTES: u64 = 2 * 1024 * 1024;

#[tauri::command]
pub fn read_file(path: String, state: State<AppState>) -> Result<String, String> {
    let target = PathBuf::from(&path);
    if !is_allowed(&target, &state) {
        return Err("That file is outside the folders AgentDeck is allowed to read.".into());
    }
    let metadata = fs::metadata(&target)
        .map_err(|error| format!("Could not read {path}: {error}"))?;
    if metadata.len() > MAX_RAW_FILE_BYTES {
        return Err(format!("{path} is larger than 2 MB and is shown by path only."));
    }
    fs::read_to_string(&target).map_err(|error| format!("Could not read {path}: {error}"))
}

fn is_allowed(target: &Path, state: &AppState) -> bool {
    let Ok(candidate) = fs::canonicalize(target) else {
        return false;
    };
    if let Some(home) = dirs_home() {
        if let Ok(home) = fs::canonicalize(home) {
            if candidate.starts_with(home) {
                return true;
            }
        }
    }
    if let Some(project) = state.project_snapshot() {
        if let Ok(project) = fs::canonicalize(project) {
            if candidate.starts_with(project) {
                return true;
            }
        }
    }
    false
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
}
