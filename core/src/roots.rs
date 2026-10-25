use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::model::Ecosystem;

#[derive(Clone, Debug)]
pub struct Roots {
    pub home: PathBuf,
    pub app_data: PathBuf,
    pub project_root: Option<PathBuf>,
    pub overrides: HashMap<Ecosystem, PathBuf>,
}

impl Roots {
    pub fn new(home: PathBuf, app_data: PathBuf, project_root: Option<PathBuf>) -> Self {
        Self {
            home,
            app_data,
            project_root,
            overrides: HashMap::new(),
        }
    }

    pub fn discover(project_root: Option<PathBuf>) -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let app_data = dirs::data_local_dir()
            .unwrap_or_else(|| home.join(".agentdeck"))
            .join("AgentDeck");
        Self::new(home, app_data, project_root)
    }

    pub fn with_override(mut self, ecosystem: Ecosystem, path: PathBuf) -> Self {
        self.overrides.insert(ecosystem, path);
        self
    }

    pub fn ecosystem_root(&self, ecosystem: Ecosystem) -> PathBuf {
        if let Some(path) = self.overrides.get(&ecosystem) {
            return path.clone();
        }
        match ecosystem {
            Ecosystem::Claude => self.home.join(".claude"),
            Ecosystem::Codex => self.home.join(".codex"),
            Ecosystem::Opencode => self.home.join(".config").join("opencode"),
            Ecosystem::CommandCode => self.home.join(".commandcode"),
        }
    }

    pub fn config_root(&self, ecosystem: Ecosystem) -> PathBuf {
        if let Some(path) = self.overrides.get(&ecosystem) {
            return path.clone();
        }
        match ecosystem {
            Ecosystem::Claude => self.home.clone(),
            Ecosystem::Codex => self.home.join(".codex"),
            Ecosystem::Opencode => self.home.join(".config").join("opencode"),
            Ecosystem::CommandCode => self.home.join(".commandcode"),
        }
    }
}

pub fn resolve_project_root(selected: &Path) -> (PathBuf, bool) {
    let mut current = Some(selected);
    while let Some(directory) = current {
        if directory.join(".git").exists() {
            return (directory.to_path_buf(), true);
        }
        current = directory.parent();
    }
    (selected.to_path_buf(), false)
}
