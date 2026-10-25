use std::path::PathBuf;

use agentdeck_core::model::ScanRequest;
use agentdeck_core::roots::Roots;
use agentdeck_core::{model::ScanResult, Manager};

#[tauri::command]
pub fn scan(request: Option<ScanRequest>) -> Result<ScanResult, String> {
    let project_root = request
        .and_then(|value| value.project_root)
        .map(PathBuf::from);
    let roots = Roots::discover(project_root);
    Ok(Manager::new(roots).scan())
}
