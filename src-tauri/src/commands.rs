use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use agentdeck_core::actions::{
    ActionPlan, ActionResult, HistoryEntry, SnapshotDiff, TrashEntry,
};
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
            roots.binaries.insert(ecosystem, PathBuf::from(value));
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

fn manager_for(state: &AppState) -> Manager {
    build_manager(state, state.project_snapshot())
}

#[tauri::command]
pub fn plan_toggle(
    artifact_id: String,
    enabled: bool,
    state: State<AppState>,
) -> Result<ActionPlan, String> {
    manager_for(&state).plan_toggle(&artifact_id, enabled)
}

#[tauri::command]
pub fn plan_remove(artifact_id: String, state: State<AppState>) -> Result<ActionPlan, String> {
    manager_for(&state).plan_remove(&artifact_id)
}

#[tauri::command]
pub fn apply_plan(plan: ActionPlan, state: State<AppState>) -> Result<ActionResult, String> {
    let manager = manager_for(&state);
    let scan = manager.apply_plan(&plan)?;
    Ok(ActionResult { plan, scan })
}

#[tauri::command]
pub fn restore(trash_id: String, state: State<AppState>) -> Result<ActionResult, String> {
    let manager = manager_for(&state);
    let entries = manager.list_trash();
    let entry = entries
        .iter()
        .find(|entry| entry.id == trash_id)
        .ok_or_else(|| "That trash entry no longer exists.".to_string())?;
    let plan = ActionPlan {
        action: agentdeck_core::actions::ActionKind::Remove,
        artifact_id: entry.artifact_id.clone(),
        artifact_name: entry.artifact_name.clone(),
        summary: format!("Restore \"{}\" from the trash.", entry.artifact_name),
        affected_ecosystems: vec![entry.ecosystem],
        files: vec![agentdeck_core::actions::PlanFile {
            path: entry.origin_path.clone(),
            operation: "move".into(),
            description: "Move the artifact back to its original path".into(),
        }],
        warnings: Vec::new(),
    };
    let scan = manager.restore(&trash_id)?;
    Ok(ActionResult { plan, scan })
}

#[tauri::command]
pub fn list_trash(state: State<AppState>) -> Vec<TrashEntry> {
    manager_for(&state).list_trash()
}

#[tauri::command]
pub fn list_history(path: Option<String>, state: State<AppState>) -> Vec<HistoryEntry> {
    manager_for(&state).list_history(path.as_deref())
}

#[tauri::command]
pub fn read_snapshot(snapshot_path: String, state: State<AppState>) -> Result<String, String> {
    manager_for(&state).read_snapshot(&snapshot_path)
}

#[tauri::command]
pub fn diff_snapshot(
    snapshot_path: String,
    state: State<AppState>,
) -> Result<SnapshotDiff, String> {
    manager_for(&state).diff_snapshot(&snapshot_path)
}

#[tauri::command]
pub fn rollback(snapshot_path: String, state: State<AppState>) -> Result<ActionResult, String> {
    let manager = manager_for(&state);
    let scan = manager.rollback(&snapshot_path)?;
    let plan = ActionPlan {
        action: agentdeck_core::actions::ActionKind::Enable,
        artifact_id: String::new(),
        artifact_name: "Rollback".into(),
        summary: "Roll the file back to a snapshot.".into(),
        affected_ecosystems: Vec::new(),
        files: Vec::new(),
        warnings: Vec::new(),
    };
    Ok(ActionResult { plan, scan })
}

#[tauri::command]
pub fn prune_snapshots(ids: Vec<String>, state: State<AppState>) -> Result<usize, String> {
    manager_for(&state).prune_snapshots(&ids)
}

#[tauri::command]
pub fn history_for_artifact(
    artifact_id: String,
    state: State<AppState>,
) -> Vec<HistoryEntry> {
    manager_for(&state).history_for_artifact(&artifact_id)
}
