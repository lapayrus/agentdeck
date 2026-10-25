pub mod jsonc;
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
            candidates.extend(source::collect(
                source,
                self.roots.project_root.as_deref(),
                &mut warnings,
            ));
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
            .any(|warning| warning.to_lowercase().contains("no description")));
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

    fn write_toml_agent(path: &Path, name: &str, description: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                "name = \"{name}\"\ndescription = \"{description}\"\ndeveloper_instructions = \"Do the work\"\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn lists_codex_agents_and_lets_the_project_definition_win() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write_toml_agent(
            &home.path().join(".codex/agents/reviewer.toml"),
            "reviewer",
            "Global reviewer",
        );
        write_toml_agent(
            &project.join(".codex/agents/reviewer.toml"),
            "reviewer",
            "Project reviewer",
        );

        let result = manager(home.path(), Some(&project)).scan();
        let reviewer: Vec<_> = result
            .artifacts
            .iter()
            .filter(|artifact| artifact.name == "reviewer")
            .collect();

        assert_eq!(reviewer.len(), 1);
        assert_eq!(reviewer[0].ecosystem, Ecosystem::Codex);
        assert_eq!(reviewer[0].scope, model::Scope::Project);
        assert_eq!(reviewer[0].shadowed.len(), 1);
    }

    #[test]
    fn codex_agents_skip_a_toml_without_a_name() {
        let home = tempdir().unwrap();
        let agents = home.path().join(".codex/agents");
        fs::create_dir_all(&agents).unwrap();
        fs::write(agents.join("broken.toml"), "description = \"No name\"\n").unwrap();

        let result = manager(home.path(), None).scan();
        assert!(result.artifacts.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("broken.toml")));
    }

    #[test]
    fn lists_opencode_agents_and_lets_the_project_definition_win() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write_agent(
            &home.path().join(".config/opencode/agents/reviewer.md"),
            "reviewer",
            Some("Global reviewer"),
        );
        write_agent(
            &project.join(".opencode/agents/reviewer.md"),
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
        assert_eq!(reviewer[0].ecosystem, Ecosystem::Opencode);
        assert_eq!(reviewer[0].scope, model::Scope::Project);
        assert_eq!(reviewer[0].shadowed.len(), 1);
    }

    #[test]
    fn opencode_agents_without_a_description_are_skipped() {
        let home = tempdir().unwrap();
        write_agent(
            &home.path().join(".config/opencode/agents/silent.md"),
            "silent",
            None,
        );

        let result = manager(home.path(), None).scan();
        assert!(result.artifacts.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("missing description")));
    }

    #[test]
    fn command_code_personal_agents_shadow_project_agents() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write_agent(
            &home.path().join(".commandcode/agents/reviewer.md"),
            "reviewer",
            Some("Personal reviewer"),
        );
        write_agent(
            &project.join(".commandcode/agents/reviewer.md"),
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
        assert_eq!(reviewer[0].ecosystem, Ecosystem::CommandCode);
        assert_eq!(reviewer[0].scope, model::Scope::Global);
        assert_eq!(reviewer[0].shadowed[0].scope, model::Scope::Project);
    }

    #[test]
    fn command_code_reserved_agent_names_are_skipped() {
        let home = tempdir().unwrap();
        write_agent(
            &home.path().join(".commandcode/agents/plan.md"),
            "plan",
            Some("Reserved name"),
        );

        let result = manager(home.path(), None).scan();
        assert!(result.artifacts.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("reserved")));
    }

    #[test]
    fn agents_loaded_from_one_path_by_several_ecosystems_appear_once() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        let shared = project.join(".agents/agents");
        write_agent(&shared.join("shared.md"), "shared", Some("Shared agent"));

        let mut claude = vec![source::Source {
            ecosystem: Ecosystem::Claude,
            artifact_type: model::ArtifactType::Agent,
            scope: model::Scope::Project,
            root: shared.clone(),
            layout: source::Layout::FilesRecursive,
            format: source::Format::MarkdownAgent,
            rank: 0,
            reload: model::ReloadHint::Live,
            disable_mechanism: None,
        }];
        claude.push(source::Source {
            ecosystem: Ecosystem::Codex,
            artifact_type: model::ArtifactType::Agent,
            scope: model::Scope::Project,
            root: shared.clone(),
            layout: source::Layout::FilesRecursive,
            format: source::Format::MarkdownAgent,
            rank: 0,
            reload: model::ReloadHint::Restart,
            disable_mechanism: None,
        });

        let mut warnings = Vec::new();
        let mut candidates = Vec::new();
        for entry in &claude {
            candidates.extend(source::collect(entry, None, &mut warnings));
        }
        let artifacts = resolve::resolve(candidates, &mut warnings);
        let shared_artifacts: Vec<_> = artifacts
            .iter()
            .filter(|artifact| artifact.name == "shared")
            .collect();

        assert_eq!(shared_artifacts.len(), 1);
        assert_eq!(shared_artifacts[0].ecosystems.len(), 2);
    }

    fn write_skill(directory: &Path, name: &str, description: Option<&str>) {
        fs::create_dir_all(directory).unwrap();
        let description = description
            .map(|value| format!("description: {value}\n"))
            .unwrap_or_default();
        fs::write(
            directory.join("SKILL.md"),
            format!("---\nname: {name}\n{description}---\n\nInstructions.\n"),
        )
        .unwrap();
    }

    #[test]
    fn a_skill_in_the_agents_root_appears_once_with_three_ecosystems() {
        let home = tempdir().unwrap();
        write_skill(
            &home.path().join(".agents/skills/shared"),
            "shared",
            Some("Shared skill"),
        );

        let result = manager(home.path(), None).scan();
        let shared: Vec<_> = result
            .artifacts
            .iter()
            .filter(|artifact| artifact.name == "shared")
            .collect();

        assert_eq!(shared.len(), 1);
        assert_eq!(shared[0].ecosystems.len(), 3);
        assert!(shared[0].ecosystems.contains(&Ecosystem::Codex));
        assert!(shared[0].ecosystems.contains(&Ecosystem::Opencode));
        assert!(shared[0].ecosystems.contains(&Ecosystem::CommandCode));
        assert_eq!(shared[0].artifact_type, model::ArtifactType::Skill);
    }

    #[test]
    fn claude_personal_skills_shadow_project_skills() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write_skill(
            &home.path().join(".claude/skills/reviewer"),
            "reviewer",
            Some("Personal reviewer"),
        );
        write_skill(
            &project.join(".claude/skills/reviewer"),
            "reviewer",
            Some("Project reviewer"),
        );

        let result = manager(home.path(), Some(&project)).scan();
        let reviewer: Vec<_> = result
            .artifacts
            .iter()
            .filter(|artifact| {
                artifact.name == "reviewer" && artifact.ecosystem == Ecosystem::Claude
            })
            .collect();

        assert_eq!(reviewer.len(), 1);
        assert_eq!(reviewer[0].scope, model::Scope::Global);
        assert_eq!(
            reviewer[0].description.as_deref(),
            Some("Personal reviewer")
        );
        assert_eq!(reviewer[0].shadowed.len(), 1);
        assert_eq!(reviewer[0].shadowed[0].scope, model::Scope::Project);
    }

    #[test]
    fn codex_skills_without_a_description_are_skipped() {
        let home = tempdir().unwrap();
        write_skill(
            &home.path().join(".agents/skills/silent"),
            "silent",
            None,
        );

        let result = manager(home.path(), None).scan();
        assert!(result
            .artifacts
            .iter()
            .all(|artifact| artifact.name != "silent"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("missing description")));
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    }

    #[test]
    fn claude_mcp_prefers_the_project_local_definition() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let project_key = project.to_string_lossy().to_string();

        write_json(
            &home.path().join(".claude.json"),
            serde_json::json!({
                "mcpServers": {
                    "shared": { "command": "global-cmd" }
                },
                "projects": {
                    project_key: {
                        "mcpServers": {
                            "shared": { "command": "local-cmd" }
                        },
                        "disabledMcpServers": []
                    }
                }
            }),
        );
        write_json(
            &project.join(".mcp.json"),
            serde_json::json!({
                "mcpServers": {
                    "shared": { "command": "project-cmd" }
                }
            }),
        );

        let result = manager(home.path(), Some(&project)).scan();
        let shared: Vec<_> = result
            .artifacts
            .iter()
            .filter(|artifact| {
                artifact.name == "shared" && artifact.ecosystem == Ecosystem::Claude
            })
            .collect();

        assert_eq!(shared.len(), 1);
        assert_eq!(shared[0].artifact_type, model::ArtifactType::Mcp);
        assert_eq!(shared[0].scope, model::Scope::ProjectLocal);
        assert_eq!(shared[0].command.as_deref(), Some("local-cmd"));
        assert_eq!(shared[0].shadowed.len(), 2);
    }

    #[test]
    fn reads_opencode_jsonc_mcp_servers_without_losing_comments() {
        let home = tempdir().unwrap();
        let path = home.path().join(".config/opencode/opencode.jsonc");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "{\n  // servers\n  \"mcp\": {\n    \"servers\": {\n      \"remote\": {\n        \"type\": \"remote\",\n        \"url\": \"https://example.test/mcp\",\n        \"disabled\": true\n      }\n    }\n  }\n}\n",
        )
        .unwrap();

        let result = manager(home.path(), None).scan();
        let remote: Vec<_> = result
            .artifacts
            .iter()
            .filter(|artifact| artifact.name == "remote")
            .collect();

        assert_eq!(remote.len(), 1);
        assert_eq!(remote[0].ecosystem, Ecosystem::Opencode);
        assert_eq!(remote[0].transport.as_deref(), Some("http"));
        assert_eq!(remote[0].enabled, Some(false));
    }

    #[test]
    fn reads_codex_mcp_servers_from_toml() {
        let home = tempdir().unwrap();
        let path = home.path().join(".codex/config.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "[mcp_servers.repl]\ncommand = \"node\"\nargs = [\"repl.js\"]\nenabled = false\n",
        )
        .unwrap();

        let result = manager(home.path(), None).scan();
        let repl: Vec<_> = result
            .artifacts
            .iter()
            .filter(|artifact| artifact.name == "repl")
            .collect();

        assert_eq!(repl.len(), 1);
        assert_eq!(repl[0].ecosystem, Ecosystem::Codex);
        assert_eq!(repl[0].command.as_deref(), Some("node repl.js"));
        assert_eq!(repl[0].enabled, Some(false));
    }

    #[test]
    fn finds_command_code_project_local_mcp_by_slug() {
        let home = tempdir().unwrap();
        let project = home.path().join("project").join("nested");
        fs::create_dir_all(&project).unwrap();
        let slug = source::project_slug(&project);
        write_json(
            &home.path()
                .join(".commandcode")
                .join("projects")
                .join(slug)
                .join("mcp.json"),
            serde_json::json!({
                "mcpServers": {
                    "local": { "command": "local-server" }
                }
            }),
        );

        let result = manager(home.path(), Some(&project)).scan();
        let local: Vec<_> = result
            .artifacts
            .iter()
            .filter(|artifact| artifact.name == "local")
            .collect();

        assert_eq!(local.len(), 1);
        assert_eq!(local[0].scope, model::Scope::ProjectLocal);
        assert_eq!(local[0].ecosystem, Ecosystem::CommandCode);
    }
}
