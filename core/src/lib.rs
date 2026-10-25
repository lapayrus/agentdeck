pub mod model;
pub mod roots;

use chrono::Utc;
use model::{Ecosystem, EcosystemDetection, ScanResult};
use roots::Roots;

pub struct Manager {
    roots: Roots,
}

impl Manager {
    pub fn new(roots: Roots) -> Self {
        Self { roots }
    }

    pub fn roots(&self) -> &Roots {
        &self.roots
    }

    pub fn scan(&self) -> ScanResult {
        let mut warnings = Vec::new();
        let detections: Vec<EcosystemDetection> = Ecosystem::ALL
            .iter()
            .map(|ecosystem| self.detect(*ecosystem))
            .collect();

        for detection in &detections {
            if detection.detected && detection.version.is_none() {
                warnings.push(format!(
                    "{} detected but its version could not be read.",
                    detection.ecosystem.label()
                ));
            }
        }

        let (project_root, is_git_repository) = match &self.roots.project_root {
            Some(root) => {
                let (resolved, is_git) = roots::resolve_project_root(root);
                (Some(resolved.to_string_lossy().to_string()), is_git)
            }
            None => (None, false),
        };

        ScanResult {
            artifacts: Vec::new(),
            detections,
            warnings,
            project_root,
            is_git_repository,
            scanned_at: Utc::now().to_rfc3339(),
        }
    }

    fn detect(&self, ecosystem: Ecosystem) -> EcosystemDetection {
        let root = self.roots.ecosystem_root(ecosystem);
        let detected = root.exists();
        EcosystemDetection {
            ecosystem,
            detected,
            version: None,
            checked_paths: vec![root.to_string_lossy().to_string()],
            capabilities: Vec::new(),
            reason: if detected {
                None
            } else {
                Some(format!("{} was not found at this path.", ecosystem.label()))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn manager(home: &std::path::Path) -> Manager {
        Manager::new(Roots::new(
            home.to_path_buf(),
            home.join("appdata"),
            None,
        ))
    }

    #[test]
    fn detects_only_ecosystems_with_a_config_root() {
        let home = tempdir().unwrap();
        fs::create_dir_all(home.path().join(".claude")).unwrap();
        fs::create_dir_all(home.path().join(".config").join("opencode")).unwrap();

        let result = manager(home.path()).scan();
        let detected: Vec<_> = result
            .detections
            .iter()
            .filter(|entry| entry.detected)
            .map(|entry| entry.ecosystem)
            .collect();

        assert_eq!(detected, vec![Ecosystem::Claude, Ecosystem::Opencode]);
    }

    #[test]
    fn reports_checked_paths_for_undetected_ecosystems() {
        let home = tempdir().unwrap();
        let result = manager(home.path()).scan();
        let codex = result
            .detections
            .iter()
            .find(|entry| entry.ecosystem == Ecosystem::Codex)
            .unwrap();

        assert!(!codex.detected);
        assert_eq!(codex.checked_paths.len(), 1);
        assert!(codex.reason.is_some());
    }

    #[test]
    fn resolves_a_non_git_project_root() {
        let home = tempdir().unwrap();
        let project = home.path().join("scratch");
        fs::create_dir_all(&project).unwrap();
        let manager = Manager::new(Roots::new(
            home.path().to_path_buf(),
            home.path().join("appdata"),
            Some(project.clone()),
        ));

        let result = manager.scan();
        assert!(!result.is_git_repository);
        assert_eq!(
            result.project_root.as_deref(),
            Some(project.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn resolves_up_to_the_git_root() {
        let home = tempdir().unwrap();
        let root = home.path().join("repo");
        let nested = root.join("src").join("deep");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(&nested).unwrap();
        let manager = Manager::new(Roots::new(
            home.path().to_path_buf(),
            home.path().join("appdata"),
            Some(nested),
        ));

        let result = manager.scan();
        assert!(result.is_git_repository);
        assert_eq!(
            result.project_root.as_deref(),
            Some(root.to_string_lossy().as_ref())
        );
    }
}
