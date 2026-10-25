pub mod model;
pub mod parser;
pub mod resolve;
pub mod roots;
pub mod source;
pub mod state;

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

        let sources = source::sources(&self.roots);
        let mut candidates = Vec::new();
        for source in &sources {
            candidates.extend(source::collect(source, &mut warnings));
        }
        let artifacts = resolve::resolve(candidates, &mut warnings);

        let (project_root, is_git_repository) = match &self.roots.project_root {
            Some(root) => {
                let (resolved, is_git) = roots::resolve_project_root(root);
                (Some(resolved.to_string_lossy().to_string()), is_git)
            }
            None => (None, false),
        };

        ScanResult {
            artifacts,
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
        let mut checked_paths = vec![root.to_string_lossy().to_string()];
        if ecosystem == Ecosystem::Claude {
            checked_paths.push(
                root.join("agents")
                    .to_string_lossy()
                    .to_string(),
            );
        }
        EcosystemDetection {
            ecosystem,
            detected,
            version: None,
            checked_paths,
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
    use std::path::Path;
    use tempfile::tempdir;

    fn manager(home: &Path, project: Option<&Path>) -> Manager {
        Manager::new(Roots::new(
            home.to_path_buf(),
            home.join("appdata"),
            project.map(|path| path.to_path_buf()),
        ))
    }

    fn write_agent(path: &Path, name: &str, description: Option<&str>) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let description = description
            .map(|value| format!("description: {value}\n"))
            .unwrap_or_default();
        fs::write(
            path,
            format!("---\nname: {name}\n{description}---\n\nInstructions.\n"),
        )
        .unwrap();
    }

    #[test]
    fn detects_only_ecosystems_with_a_config_root() {
        let home = tempdir().unwrap();
        fs::create_dir_all(home.path().join(".claude")).unwrap();
        fs::create_dir_all(home.path().join(".config").join("opencode")).unwrap();

        let result = manager(home.path(), None).scan();
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
        let result = manager(home.path(), None).scan();
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
        let result = manager(home.path(), Some(&project)).scan();

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
        let result = manager(home.path(), Some(&nested)).scan();

        assert!(result.is_git_repository);
        assert_eq!(
            result.project_root.as_deref(),
            Some(root.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn lists_global_and_project_agents() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write_agent(
            &home.path().join(".claude/agents/global-agent.md"),
            "global-agent",
            Some("Lives globally"),
        );
        write_agent(
            &project.join(".claude/agents/project-agent.md"),
            "project-agent",
            Some("Lives in the project"),
        );

        let result = manager(home.path(), Some(&project)).scan();
        let names: Vec<&str> = result
            .artifacts
            .iter()
            .map(|artifact| artifact.name.as_str())
            .collect();

        assert!(names.contains(&"global-agent"));
        assert!(names.contains(&"project-agent"));
        let project_agent = result
            .artifacts
            .iter()
            .find(|artifact| artifact.name == "project-agent")
            .unwrap();
        assert_eq!(project_agent.scope, model::Scope::Project);
    }

    #[test]
    fn project_definition_shadows_the_global_one() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write_agent(
            &home.path().join(".claude/agents/reviewer.md"),
            "reviewer",
            Some("Global reviewer"),
        );
        write_agent(
            &project.join(".claude/agents/reviewer.md"),
            "reviewer",
            Some("Project reviewer"),
        );

        let result = manager(home.path(), Some(&project)).scan();
        let reviewer: Vec<_> = result
            .artifacts
            .iter()
            .filter(|artifact| artifact.name == "reviewer")
            .collect();

        assert_eq!(reviewer.len(), 1);
        assert_eq!(reviewer[0].scope, model::Scope::Project);
        assert_eq!(reviewer[0].description.as_deref(), Some("Project reviewer"));
        assert_eq!(reviewer[0].shadowed.len(), 1);
        assert_eq!(reviewer[0].shadowed[0].scope, model::Scope::Global);
    }

    #[test]
    fn a_malformed_file_warns_without_hiding_other_agents() {
        let home = tempdir().unwrap();
        let agents = home.path().join(".claude/agents");
        fs::create_dir_all(&agents).unwrap();
        fs::write(agents.join("broken.md"), "---\nname: broken\n").unwrap();
        write_agent(&agents.join("good.md"), "good", Some("Loads fine"));

        let result = manager(home.path(), None).scan();

        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("broken.md")));
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].name, "good");
    }

    #[test]
    fn an_agent_without_a_description_carries_a_warning() {
        let home = tempdir().unwrap();
        write_agent(
            &home.path().join(".claude/agents/silent.md"),
            "silent",
            None,
        );

        let result = manager(home.path(), None).scan();
        assert_eq!(result.artifacts.len(), 1);
        assert!(result.artifacts[0]
            .warnings
            .iter()
            .any(|warning| warning.contains("no description")));
    }

    #[test]
    fn scans_agents_from_ancestors_up_to_the_git_root() {
        let home = tempdir().unwrap();
        let root = home.path().join("repo");
        let nested = root.join("packages").join("app");
        fs::create_dir_all(root.join(".git")).unwrap();
        write_agent(
            &root.join(".claude/agents/root-agent.md"),
            "root-agent",
            Some("At the git root"),
        );
        write_agent(
            &nested.join(".claude/agents/nested-agent.md"),
            "nested-agent",
            Some("At the nested folder"),
        );

        let result = manager(home.path(), Some(&nested)).scan();
        let names: Vec<&str> = result
            .artifacts
            .iter()
            .map(|artifact| artifact.name.as_str())
            .collect();

        assert!(names.contains(&"root-agent"));
        assert!(names.contains(&"nested-agent"));
        let nested_agent = result
            .artifacts
            .iter()
            .find(|artifact| artifact.name == "nested-agent")
            .unwrap();
        assert_eq!(nested_agent.scope, model::Scope::Project);
    }
}
