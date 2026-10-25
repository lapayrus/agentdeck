use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use similar::TextDiff;
use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

use crate::hash::hash_text;
use crate::jsonc_edit;
use crate::model::{Artifact, ArtifactType, Ecosystem, Scope, ScanResult};
use crate::state::write_atomic;
use crate::Manager;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ActionKind {
    Disable,
    Enable,
    Remove,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlanFile {
    pub path: String,
    pub operation: String,
    pub description: String,
    pub expected_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionPlan {
    pub action: ActionKind,
    pub artifact_id: String,
    pub artifact_name: String,
    pub summary: String,
    pub affected_ecosystems: Vec<Ecosystem>,
    pub files: Vec<PlanFile>,
    pub warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub plan: ActionPlan,
    pub scan: ScanResult,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub path: String,
    pub action: String,
    pub timestamp: String,
    pub ecosystem: Option<Ecosystem>,
    pub artifact_name: Option<String>,
    pub snapshot_path: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrashEntry {
    pub id: String,
    pub artifact_id: String,
    pub artifact_name: String,
    pub artifact_type: ArtifactType,
    pub ecosystem: Ecosystem,
    pub scope: Scope,
    pub origin_path: String,
    pub trashed_path: String,
    pub timestamp: String,
    pub kind: String,
    pub restored: bool,
    pub snapshot_path: Option<String>,
    pub after_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDiff {
    pub path: String,
    pub snapshot: String,
    pub current: String,
    pub diff: String,
}

fn timestamp() -> String {
    Utc::now().to_rfc3339()
}

fn short_id(value: &str) -> String {
    let hash = hash_text(value);
    format!("{}-{}", Utc::now().timestamp_millis(), &hash[..8])
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '.' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn read_manifest<T: for<'de> Deserialize<'de>>(path: &Path) -> Vec<T> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write_manifest<T: Serialize>(path: &Path, entries: &[T]) -> Result<(), String> {
    let text = serde_json::to_string_pretty(entries)
        .map_err(|error| format!("Could not serialize the manifest: {error}"))?;
    write_atomic(path, &text)
}

fn move_path(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) => {
            if source.is_dir() {
                copy_directory(source, destination)?;
                fs::remove_dir_all(source)
                    .map_err(|error| format!("Could not remove {}: {error}", source.display()))
            } else {
                fs::copy(source, destination)
                    .map_err(|error| format!("Could not copy {}: {error}", source.display()))?;
                fs::remove_file(source)
                    .map_err(|error| format!("Could not remove {}: {error}", source.display()))
            }
        }
    }
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("Could not create {}: {error}", destination.display()))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("Could not read {}: {error}", source.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

impl Manager {
    fn history_dir(&self) -> PathBuf {
        self.roots().app_data.join("history")
    }

    fn history_manifest(&self) -> PathBuf {
        self.history_dir().join("manifest.json")
    }

    fn trash_dir(&self) -> PathBuf {
        self.roots().app_data.join("trash")
    }

    fn trash_manifest(&self) -> PathBuf {
        self.trash_dir().join("manifest.json")
    }

    fn quarantine_dir(&self) -> PathBuf {
        self.roots().app_data.join("quarantine")
    }

    fn quarantine_manifest(&self) -> PathBuf {
        self.quarantine_dir().join("manifest.json")
    }

    fn project_root(&self) -> Result<PathBuf, String> {
        self.roots()
            .project_root
            .clone()
            .ok_or_else(|| "Select a project before changing project-scoped artifacts.".to_string())
    }

    fn claude_settings(&self, scope: Scope) -> Result<PathBuf, String> {
        if scope == Scope::Global {
            Ok(self.roots().home.join(".claude").join("settings.json"))
        } else {
            Ok(self
                .project_root()?
                .join(".claude")
                .join("settings.local.json"))
        }
    }

    fn commandcode_settings(&self, scope: Scope) -> Result<PathBuf, String> {
        if scope == Scope::Global {
            Ok(self.roots().home.join(".commandcode").join("settings.json"))
        } else {
            Ok(self
                .project_root()?
                .join(".commandcode")
                .join("settings.json"))
        }
    }

    fn codex_config(&self, scope: Scope) -> Result<PathBuf, String> {
        if scope == Scope::Global {
            Ok(self.roots().home.join(".codex").join("config.toml"))
        } else {
            Ok(self.project_root()?.join(".codex").join("config.toml"))
        }
    }

    fn opencode_config(&self, scope: Scope) -> Result<PathBuf, String> {
        if scope == Scope::Global {
            Ok(self
                .roots()
                .home
                .join(".config")
                .join("opencode")
                .join("opencode.json"))
        } else {
            Ok(self.project_root()?.join("opencode.json"))
        }
    }

    fn find_artifact(&self, id: &str) -> Result<Artifact, String> {
        self.scan()
            .artifacts
            .into_iter()
            .find(|artifact| artifact.id == id)
            .ok_or_else(|| "That artifact is no longer present; rescan and try again.".to_string())
    }

    fn snapshot(
        &self,
        path: &Path,
        action: &str,
        artifact: &Artifact,
    ) -> Result<HistoryEntry, String> {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        let hash = hash_text(&text);
        let id = short_id(&format!("{}-{action}", path.display()));
        let file_name = sanitize(
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("file"),
        );
        fs::create_dir_all(self.history_dir())
            .map_err(|error| format!("Could not create history: {error}"))?;
        let snapshot_path = self.history_dir().join(format!("{id}-{file_name}"));
        write_atomic(&snapshot_path, &text)?;

        let entry = HistoryEntry {
            id,
            path: path.to_string_lossy().to_string(),
            action: action.to_string(),
            timestamp: timestamp(),
            ecosystem: Some(artifact.ecosystem),
            artifact_name: Some(artifact.name.clone()),
            snapshot_path: snapshot_path.to_string_lossy().to_string(),
            hash,
        };
        let mut entries: Vec<HistoryEntry> = read_manifest(&self.history_manifest());
        entries.push(entry.clone());
        write_manifest(&self.history_manifest(), &entries)?;
        Ok(entry)
    }

    fn edit_file<F>(
        &self,
        path: &Path,
        artifact: &Artifact,
        action: &str,
        expected: &HashMap<String, String>,
        default_contents: &str,
        edit: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        let existed = path.is_file();
        let text = if existed {
            fs::read_to_string(path)
                .map_err(|error| format!("Could not read {}: {error}", path.display()))?
        } else {
            default_contents.to_string()
        };
        if existed {
            let path_key = path.to_string_lossy().to_string();
            if let Some(expected_hash) = expected.get(&path_key) {
                if &hash_text(&text) != expected_hash {
                    return Err(format!(
                        "{} changed since the action was planned. Rescan before trying again.",
                        path.display()
                    ));
                }
            }
            if path == Path::new(&artifact.primary_path) {
                if let Some(expected_hash) = &artifact.content_hash {
                    if &hash_text(&text) != expected_hash {
                        return Err(format!(
                            "{} changed since the last scan. Rescan before trying again.",
                            path.display()
                        ));
                    }
                }
            }
        }
        if existed {
            self.snapshot(path, action, artifact)?;
        }
        let updated = edit(&text)?;
        if updated != text || !existed {
            write_atomic(path, &updated)?;
        }
        Ok(())
    }

    fn toggle_ecosystem(
        &self,
        artifact: &Artifact,
        ecosystem: Ecosystem,
        enable: bool,
        expected: &HashMap<String, String>,
    ) -> Result<(), String> {
        let action = if enable { "enable" } else { "disable" };
        match (artifact.artifact_type, ecosystem) {
            (ArtifactType::Agent, Ecosystem::Claude) => {
                let path = self.claude_settings(artifact.scope)?;
                let value = serde_json::to_string(&format!("Agent({})", artifact.name))
                    .map_err(|error| error.to_string())?;
                self.edit_file(&path, artifact, action, expected, "{}", |text| {
                    let text = jsonc_edit::ensure_object(text, &["permissions"])?;
                    if enable {
                        jsonc_edit::remove_array_value(&text, &["permissions", "deny"], &value)
                    } else {
                        jsonc_edit::ensure_array_contains(&text, &["permissions", "deny"], &value)
                    }
                })
            }
            (ArtifactType::Agent, Ecosystem::Opencode) => {
                if enable {
                    self.restore_quarantined(artifact)
                } else {
                    self.quarantine(artifact)
                }
            }
            (ArtifactType::Agent, Ecosystem::Codex)
            | (ArtifactType::Agent, Ecosystem::CommandCode) => {
                if enable {
                    self.restore_quarantined(artifact)
                } else {
                    self.quarantine(artifact)
                }
            }
            (ArtifactType::Skill, Ecosystem::Claude) => {
                let path = self.claude_settings(artifact.scope)?;
                self.edit_file(&path, artifact, action, expected, "{}", |text| {
                    if enable {
                        let root = jsonc_edit::parse(text)?;
                        if jsonc_edit::find(&root, &["skillOverrides"]).is_none() {
                            return Ok(text.to_string());
                        }
                        jsonc_edit::remove_member(text, &["skillOverrides"], &artifact.name)
                    } else {
                        let text = jsonc_edit::ensure_object(text, &["skillOverrides"])?;
                        jsonc_edit::set_member(
                            &text,
                            &["skillOverrides"],
                            &artifact.name,
                            "\"off\"",
                        )
                    }
                })
            }
            (ArtifactType::Skill, Ecosystem::Codex) => {
                let path = self.codex_config(artifact.scope)?;
                let skill_path = artifact
                    .source_paths
                    .first()
                    .cloned()
                    .unwrap_or_else(|| artifact.primary_path.clone());
                self.edit_file(&path, artifact, action, expected, "", |text| {
                    let mut document = if text.trim().is_empty() {
                        DocumentMut::new()
                    } else {
                        text.parse::<DocumentMut>()
                            .map_err(|error| format!("invalid TOML: {error}"))?
                    };
                    let skills = document
                        .entry("skills")
                        .or_insert(Item::Table(Table::new()))
                        .as_table_mut()
                        .ok_or("The skills key is not a table.")?;
                    if !skills.contains_key("config") {
                        skills.insert("config", Item::ArrayOfTables(ArrayOfTables::new()));
                    }
                    let config = skills
                        .get_mut("config")
                        .and_then(|item| item.as_array_of_tables_mut())
                        .ok_or("The skills.config key is not an array of tables.")?;
                    if enable {
                        for entry in config.iter_mut() {
                            if entry.get("path").and_then(|item| item.as_str())
                                == Some(skill_path.as_str())
                            {
                                entry["enabled"] = value(true);
                            }
                        }
                    } else {
                        let mut entry = Table::new();
                        entry["path"] = value(skill_path.clone());
                        entry["enabled"] = value(false);
                        config.push(entry);
                    }
                    Ok(document.to_string())
                })
            }
            (ArtifactType::Skill, Ecosystem::Opencode) => {
                let path = self.opencode_config(artifact.scope)?;
                let permission = format!(
                    "{{\"action\":\"skill\",\"resource\":{},\"effect\":\"deny\"}}",
                    serde_json::to_string(&artifact.name).map_err(|error| error.to_string())?
                );
                self.edit_file(&path, artifact, action, expected, "{}", |text| {
                    if enable {
                        jsonc_edit::remove_array_value(
                            text,
                            &["permissions"],
                            &permission,
                        )
                    } else {
                        jsonc_edit::ensure_array_contains(
                            text,
                            &["permissions"],
                            &permission,
                        )
                    }
                })
            }
            (ArtifactType::Skill, Ecosystem::CommandCode) => {
                let path = self.commandcode_settings(artifact.scope)?;
                let name = serde_json::to_string(&artifact.name)
                    .map_err(|error| error.to_string())?;
                self.edit_file(&path, artifact, action, expected, "{}", |text| {
                    if enable {
                        jsonc_edit::remove_array_value(text, &["disabledSkills"], &name)
                    } else {
                        jsonc_edit::ensure_array_contains(text, &["disabledSkills"], &name)
                    }
                })
            }
            (ArtifactType::Mcp, Ecosystem::Claude) => {
                let name = serde_json::to_string(&artifact.name)
                    .map_err(|error| error.to_string())?;
                if artifact.scope == Scope::Project
                    && artifact.primary_path.ends_with(".mcp.json")
                {
                    let path = self
                        .project_root()?
                        .join(".claude")
                        .join("settings.local.json");
                    self.edit_file(&path, artifact, action, expected, "{}", |text| {
                        if enable {
                            jsonc_edit::remove_array_value(
                                text,
                                &["disabledMcpjsonServers"],
                                &name,
                            )
                        } else {
                            jsonc_edit::ensure_array_contains(
                                text,
                                &["disabledMcpjsonServers"],
                                &name,
                            )
                        }
                    })
                } else {
                    let path = self.roots().home.join(".claude.json");
                    let project = self.project_root()?;
                    let key = project.to_string_lossy().to_string();
                    self.edit_file(&path, artifact, action, expected, "{}", |text| {
                        let text = jsonc_edit::ensure_object(text, &["projects"])?;
                        let text = jsonc_edit::ensure_object(&text, &["projects", &key])?;
                        if enable {
                            jsonc_edit::remove_array_value(
                                &text,
                                &["projects", &key, "disabledMcpServers"],
                                &name,
                            )
                        } else {
                            jsonc_edit::ensure_array_contains(
                                &text,
                                &["projects", &key, "disabledMcpServers"],
                                &name,
                            )
                        }
                    })
                }
            }
            (ArtifactType::Mcp, Ecosystem::Codex) => {
                let path = PathBuf::from(&artifact.primary_path);
                self.edit_file(&path, artifact, action, expected, "", |text| {
                    let mut document = text
                        .parse::<DocumentMut>()
                        .map_err(|error| format!("invalid TOML: {error}"))?;
                    let servers = document
                        .get_mut("mcp_servers")
                        .and_then(|item| item.as_table_like_mut())
                        .ok_or("mcp_servers is not a table.")?;
                    let entry = servers
                        .get_mut(&artifact.name)
                        .and_then(|item| item.as_table_like_mut())
                        .ok_or("The server table was not found.")?;
                    entry.insert("enabled", value(enable));
                    Ok(document.to_string())
                })
            }
            (ArtifactType::Mcp, Ecosystem::Opencode) => {
                let path = PathBuf::from(&artifact.primary_path);
                self.edit_file(&path, artifact, action, expected, "", |text| {
                    let segments = self.mcp_key_segments(artifact)?;
                    let refs: Vec<&str> = segments.iter().map(|value| value.as_str()).collect();
                    jsonc_edit::set_member(text, &refs, "disabled", if enable { "false" } else { "true" })
                })
            }
            (ArtifactType::Mcp, Ecosystem::CommandCode) => {
                let path = PathBuf::from(&artifact.primary_path);
                self.edit_file(&path, artifact, action, expected, "", |text| {
                    jsonc_edit::set_member(
                        text,
                        &["mcpServers", &artifact.name],
                        "enabled",
                        if enable { "true" } else { "false" },
                    )
                })
            }
            _ => Err(format!(
                "{} cannot be changed for {}.",
                artifact.name,
                ecosystem.label()
            )),
        }
    }

    fn mcp_key_segments(&self, artifact: &Artifact) -> Result<Vec<String>, String> {
        if artifact.ecosystem == Ecosystem::Claude {
            if artifact.scope == Scope::ProjectLocal {
                let project = self.project_root()?;
                return Ok(vec![
                    "projects".into(),
                    project.to_string_lossy().to_string(),
                    "mcpServers".into(),
                    artifact.name.clone(),
                ]);
            }
            return Ok(vec!["mcpServers".into(), artifact.name.clone()]);
        }
        Ok(mcp_key_segments_error(artifact))
    }

    pub fn plan_toggle(&self, artifact_id: &str, enable: bool) -> Result<ActionPlan, String> {
        let artifact = self.find_artifact(artifact_id)?;
        if !artifact.managed {
            return Err("This artifact is read-only in AgentDeck.".into());
        }
        let action = if enable {
            ActionKind::Enable
        } else {
            ActionKind::Disable
        };
        let mut warnings = Vec::new();
        let mut files = Vec::new();
        let mut seen = HashMap::new();
        for ecosystem in &artifact.ecosystems {
            let (path, operation, description) =
                self.describe_target(&artifact, *ecosystem, enable)?;
            let key = format!("{}|{}", path, operation);
            if seen.insert(key, ()).is_none() {
                files.push(PlanFile {
                    expected_hash: crate::hash::hash_file(Path::new(&path)),
                    path,
                    operation,
                    description,
                });
            }
            if *ecosystem == Ecosystem::Codex && artifact.artifact_type == ArtifactType::Agent {
                warnings.push(
                    "Codex has no per-agent disable switch; the file moves to quarantine."
                        .into(),
                );
            }
            if *ecosystem == Ecosystem::CommandCode
                && artifact.artifact_type == ArtifactType::Agent
            {
                warnings.push(
                    "Command Code has no per-agent disable switch; the file moves to quarantine."
                        .into(),
                );
            }
        }
        let verb = if enable { "Enable" } else { "Disable" };
        let ecosystems: Vec<String> = artifact
            .ecosystems
            .iter()
            .map(|ecosystem| ecosystem.label().to_string())
            .collect();
        Ok(ActionPlan {
            action,
            artifact_id: artifact.id.clone(),
            artifact_name: artifact.name.clone(),
            summary: format!(
                "{verb} \"{}\" for {}.",
                artifact.name,
                ecosystems.join(", ")
            ),
            affected_ecosystems: artifact.ecosystems.clone(),
            files,
            warnings,
        })
    }

    fn describe_target(
        &self,
        artifact: &Artifact,
        ecosystem: Ecosystem,
        enable: bool,
    ) -> Result<(String, String, String), String> {
        let (path, description) = match (artifact.artifact_type, ecosystem) {
            (ArtifactType::Agent, Ecosystem::Claude) => (
                self.claude_settings(artifact.scope)?,
                "Add or remove the Agent deny rule".to_string(),
            ),
            (ArtifactType::Agent, Ecosystem::Opencode) => (
                PathBuf::from(&artifact.primary_path),
                if enable {
                    "Move the agent file back from quarantine".to_string()
                } else {
                    "Move the agent file to quarantine (its native flag is unverified)"
                        .to_string()
                },
            ),
            (ArtifactType::Agent, Ecosystem::Codex)
            | (ArtifactType::Agent, Ecosystem::CommandCode) => (
                PathBuf::from(&artifact.primary_path),
                if enable {
                    "Move the file back from quarantine".to_string()
                } else {
                    "Move the file to quarantine".to_string()
                },
            ),
            (ArtifactType::Skill, Ecosystem::Claude) => (
                self.claude_settings(artifact.scope)?,
                "Set or clear the skill override".to_string(),
            ),
            (ArtifactType::Skill, Ecosystem::Codex) => (
                self.codex_config(artifact.scope)?,
                "Set the skills.config entry".to_string(),
            ),
            (ArtifactType::Skill, Ecosystem::Opencode) => (
                self.opencode_config(artifact.scope)?,
                "Add or remove the skill permission".to_string(),
            ),
            (ArtifactType::Skill, Ecosystem::CommandCode) => (
                self.commandcode_settings(artifact.scope)?,
                "Add or remove the disabled-skills entry".to_string(),
            ),
            (ArtifactType::Mcp, Ecosystem::Claude) => {
                if artifact.scope == Scope::Project
                    && artifact.primary_path.ends_with(".mcp.json")
                {
                    (
                        self.project_root()?
                            .join(".claude")
                            .join("settings.local.json"),
                        "Add or remove the server from disabledMcpjsonServers".to_string(),
                    )
                } else {
                    (
                        self.roots().home.join(".claude.json"),
                        "Add or remove the server from disabledMcpServers".to_string(),
                    )
                }
            }
            (ArtifactType::Mcp, Ecosystem::Codex) => (
                PathBuf::from(&artifact.primary_path),
                "Set the server enabled flag".to_string(),
            ),
            (ArtifactType::Mcp, Ecosystem::Opencode) => (
                PathBuf::from(&artifact.primary_path),
                "Set the server disabled flag".to_string(),
            ),
            (ArtifactType::Mcp, Ecosystem::CommandCode) => (
                PathBuf::from(&artifact.primary_path),
                "Set the server enabled flag".to_string(),
            ),
            _ => {
                return Err(format!(
                    "{} cannot be changed for {}.",
                    artifact.name,
                    ecosystem.label()
                ))
            }
        };
        let moving = matches!(
            (artifact.artifact_type, ecosystem),
            (ArtifactType::Agent, Ecosystem::Codex)
                | (ArtifactType::Agent, Ecosystem::CommandCode)
                | (ArtifactType::Agent, Ecosystem::Opencode)
        );
        let operation = if moving { "move" } else { "update" };
        Ok((
            path.to_string_lossy().to_string(),
            operation.to_string(),
            description,
        ))
    }

    pub fn plan_remove(&self, artifact_id: &str) -> Result<ActionPlan, String> {
        let artifact = self.find_artifact(artifact_id)?;
        if !artifact.managed {
            return Err("This artifact is read-only in AgentDeck.".into());
        }
        let target = PathBuf::from(&artifact.primary_path);
        let (operation, description) = if artifact.artifact_type == ArtifactType::Mcp {
            (
                "update",
                "Remove the server key after snapshotting the file".to_string(),
            )
        } else {
            ("move", "Move the artifact to the app trash".to_string())
        };
        Ok(ActionPlan {
            action: ActionKind::Remove,
            artifact_id: artifact.id.clone(),
            artifact_name: artifact.name.clone(),
            summary: format!(
                "Remove \"{}\" for {}. It can be restored from the trash.",
                artifact.name,
                artifact
                    .ecosystems
                    .iter()
                    .map(|ecosystem| ecosystem.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            affected_ecosystems: artifact.ecosystems.clone(),
            files: vec![PlanFile {
                expected_hash: crate::hash::hash_file(Path::new(&target)),
                path: target.to_string_lossy().to_string(),
                operation: operation.to_string(),
                description,
            }],
            warnings: Vec::new(),
        })
    }

    pub fn apply_plan(&self, plan: &ActionPlan) -> Result<ScanResult, String> {
        let artifact = self.find_artifact(&plan.artifact_id)?;
        let expected: HashMap<String, String> = plan
            .files
            .iter()
            .filter_map(|file| {
                file.expected_hash
                    .clone()
                    .map(|hash| (file.path.clone(), hash))
            })
            .collect();
        match plan.action {
            ActionKind::Disable | ActionKind::Enable => {
                let enable = plan.action == ActionKind::Enable;
                for ecosystem in artifact.ecosystems.clone() {
                    self.toggle_ecosystem(&artifact, ecosystem, enable, &expected)?;
                }
            }
            ActionKind::Remove => self.remove_artifact(&artifact)?,
        }
        Ok(self.scan())
    }

    fn quarantine(&self, artifact: &Artifact) -> Result<(), String> {
        let origin = PathBuf::from(&artifact.primary_path);
        if !origin.exists() {
            return Err("The artifact is no longer on disk; rescan.".into());
        }
        if let Some(expected) = &artifact.content_hash {
            if crate::hash::hash_file(&origin).as_deref() != Some(expected.as_str()) {
                return Err("The artifact changed since the last scan. Rescan.".into());
            }
        }
        self.snapshot(&origin, "disable", artifact)?;
        let id = short_id(&format!("{}:{}", artifact.id, origin.display()));
        let file_name = origin
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("artifact");
        let destination = self
            .quarantine_dir()
            .join(ecosystem_key(artifact.ecosystem))
            .join(scope_key(artifact.scope))
            .join(format!("{id}-{file_name}"));
        move_path(&origin, &destination)?;

        let mut entries: Vec<TrashEntry> = read_manifest(&self.quarantine_manifest());
        entries.push(TrashEntry {
            id,
            artifact_id: artifact.id.clone(),
            artifact_name: artifact.name.clone(),
            artifact_type: artifact.artifact_type,
            ecosystem: artifact.ecosystem,
            scope: artifact.scope,
            origin_path: origin.to_string_lossy().to_string(),
            trashed_path: destination.to_string_lossy().to_string(),
            timestamp: timestamp(),
            kind: "quarantine".into(),
            restored: false,
            snapshot_path: None,
            after_hash: None,
        });
        write_manifest(&self.quarantine_manifest(), &entries)
    }

    fn restore_quarantined(&self, artifact: &Artifact) -> Result<(), String> {
        let mut entries: Vec<TrashEntry> = read_manifest(&self.quarantine_manifest());
        let Some(entry) = entries.iter_mut().find(|entry| {
            entry.kind == "quarantine" && !entry.restored && entry.artifact_id == artifact.id
        }) else {
            return Err("No quarantine entry was found for this artifact.".into());
        };
        if Path::new(&entry.origin_path).exists() {
            return Err("The original path already exists; restore was cancelled.".into());
        }
        move_path(Path::new(&entry.trashed_path), Path::new(&entry.origin_path))?;
        entry.restored = true;
        write_manifest(&self.quarantine_manifest(), &entries)
    }

    fn remove_artifact(&self, artifact: &Artifact) -> Result<(), String> {
        if artifact.artifact_type == ArtifactType::Mcp {
            return self.remove_config_key(artifact);
        }
        let primary = PathBuf::from(&artifact.primary_path);
        let origin = if artifact.artifact_type == ArtifactType::Skill {
            primary
                .parent()
                .ok_or_else(|| "The skill directory could not be resolved.".to_string())?
                .to_path_buf()
        } else {
            primary.clone()
        };
        if !origin.exists() {
            return Err("The artifact is no longer on disk; rescan.".into());
        }
        if let Some(expected) = &artifact.content_hash {
            if crate::hash::hash_file(&primary).as_deref() != Some(expected.as_str()) {
                return Err("The artifact changed since the last scan. Rescan.".into());
            }
        }
        if origin.is_file() {
            self.snapshot(&origin, "remove", artifact)?;
        } else {
            self.snapshot(&primary, "remove", artifact)?;
        }
        let id = short_id(&format!("{}:{}", artifact.id, origin.display()));
        let file_name = origin
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("artifact");
        let destination = self
            .trash_dir()
            .join(ecosystem_key(artifact.ecosystem))
            .join(scope_key(artifact.scope))
            .join(format!("{id}-{file_name}"));
        move_path(&origin, &destination)?;

        let mut entries: Vec<TrashEntry> = read_manifest(&self.trash_manifest());
        entries.push(TrashEntry {
            id,
            artifact_id: artifact.id.clone(),
            artifact_name: artifact.name.clone(),
            artifact_type: artifact.artifact_type,
            ecosystem: artifact.ecosystem,
            scope: artifact.scope,
            origin_path: origin.to_string_lossy().to_string(),
            trashed_path: destination.to_string_lossy().to_string(),
            timestamp: timestamp(),
            kind: "file".into(),
            restored: false,
            snapshot_path: None,
            after_hash: None,
        });
        write_manifest(&self.trash_manifest(), &entries)
    }

    fn remove_config_key(&self, artifact: &Artifact) -> Result<(), String> {
        let path = PathBuf::from(&artifact.primary_path);
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        if let Some(expected) = &artifact.content_hash {
            if &hash_text(&text) != expected {
                return Err("The configuration changed since the last scan. Rescan.".into());
            }
        }
        let snapshot = self.snapshot(&path, "remove", artifact)?;

        let updated = if path.extension().and_then(|value| value.to_str()) == Some("toml") {
            let mut document = text
                .parse::<DocumentMut>()
                .map_err(|error| format!("invalid TOML: {error}"))?;
            let servers = document
                .get_mut("mcp_servers")
                .and_then(|item| item.as_table_like_mut())
                .ok_or("mcp_servers is not a table.")?;
            if servers.remove(&artifact.name).is_none() {
                return Err("The server is no longer in that file.".into());
            }
            document.to_string()
        } else {
            let segments = self.mcp_key_segments(artifact)?;
            let refs: Vec<&str> = segments.iter().map(|value| value.as_str()).collect();
            let (key, parent) = refs.split_last().ok_or("Missing key path")?;
            jsonc_edit::remove_member(&text, parent, key)?
        };
        write_atomic(&path, &updated)?;

        let id = short_id(&format!("{}:{}", artifact.id, path.display()));
        let mut entries: Vec<TrashEntry> = read_manifest(&self.trash_manifest());
        entries.push(TrashEntry {
            id,
            artifact_id: artifact.id.clone(),
            artifact_name: artifact.name.clone(),
            artifact_type: artifact.artifact_type,
            ecosystem: artifact.ecosystem,
            scope: artifact.scope,
            origin_path: path.to_string_lossy().to_string(),
            trashed_path: String::new(),
            timestamp: timestamp(),
            kind: "configKey".into(),
            restored: false,
            snapshot_path: Some(snapshot.snapshot_path),
            after_hash: Some(hash_text(&updated)),
        });
        write_manifest(&self.trash_manifest(), &entries)
    }

    pub fn list_trash(&self) -> Vec<TrashEntry> {
        let mut entries: Vec<TrashEntry> = read_manifest(&self.trash_manifest());
        entries.extend(read_manifest::<TrashEntry>(&self.quarantine_manifest()));
        entries.sort_by(|left, right| left.timestamp.cmp(&right.timestamp));
        entries
    }

    pub fn prune_trash(&self, ids: &[String]) -> Result<usize, String> {
        let mut removed = 0;
        for manifest in [self.trash_manifest(), self.quarantine_manifest()] {
            let mut entries: Vec<TrashEntry> = read_manifest(&manifest);
            entries.retain(|entry| {
                if entry.restored && ids.contains(&entry.id) {
                    if !entry.trashed_path.is_empty() {
                        let path = Path::new(&entry.trashed_path);
                        if path.exists() {
                            if path.is_dir() {
                                let _ = fs::remove_dir_all(path);
                            } else {
                                let _ = fs::remove_file(path);
                            }
                        }
                    }
                    removed += 1;
                    false
                } else {
                    true
                }
            });
            write_manifest(&manifest, &entries)?;
        }
        Ok(removed)
    }

    pub fn restore(&self, trash_id: &str) -> Result<ScanResult, String> {
        enum Bucket {
            Trash,
            Quarantine,
        }
        let mut trash: Vec<TrashEntry> = read_manifest(&self.trash_manifest());
        let mut quarantine: Vec<TrashEntry> = read_manifest(&self.quarantine_manifest());
        let bucket = if trash.iter().any(|entry| entry.id == trash_id) {
            Bucket::Trash
        } else if quarantine.iter().any(|entry| entry.id == trash_id) {
            Bucket::Quarantine
        } else {
            return Err("That trash entry no longer exists.".into());
        };
        let entries = match bucket {
            Bucket::Trash => &mut trash,
            Bucket::Quarantine => &mut quarantine,
        };
        let entry = entries
            .iter_mut()
            .find(|entry| entry.id == trash_id)
            .ok_or_else(|| "That trash entry no longer exists.".to_string())?;
        if entry.restored {
            return Err("That artifact has already been restored.".into());
        }
        let origin = PathBuf::from(&entry.origin_path);
        if entry.kind == "configKey" {
            let current = crate::hash::hash_file(&origin)
                .ok_or_else(|| "The configuration file is missing.".to_string())?;
            if Some(&current) != entry.after_hash.as_ref() {
                return Err(
                    "The configuration changed after removal, so restoring would overwrite it."
                        .into(),
                );
            }
            let snapshot_path = entry
                .snapshot_path
                .clone()
                .ok_or_else(|| "The snapshot is missing.".to_string())?;
            let contents = fs::read_to_string(&snapshot_path)
                .map_err(|error| format!("Could not read the snapshot: {error}"))?;
            write_atomic(&origin, &contents)?;
        } else {
            if origin.exists() {
                return Err("The original path already exists; restore was cancelled.".into());
            }
            move_path(Path::new(&entry.trashed_path), &origin)?;
        }
        entry.restored = true;
        match bucket {
            Bucket::Trash => write_manifest(&self.trash_manifest(), &trash),
            Bucket::Quarantine => write_manifest(&self.quarantine_manifest(), &quarantine),
        }?;
        Ok(self.scan())
    }

    pub fn list_history(&self, path: Option<&str>) -> Vec<HistoryEntry> {
        let entries: Vec<HistoryEntry> = read_manifest(&self.history_manifest());
        let normalize = |value: &str| value.replace('\\', "/").to_lowercase();
        match path {
            Some(path) => {
                let target = normalize(path);
                entries
                    .into_iter()
                    .filter(|entry| normalize(&entry.path) == target)
                    .collect()
            }
            None => entries,
        }
    }

    pub fn history_for_artifact(&self, artifact_id: &str) -> Vec<HistoryEntry> {
        let path = self
            .scan()
            .artifacts
            .into_iter()
            .find(|artifact| artifact.id == artifact_id)
            .map(|artifact| artifact.primary_path);
        match path {
            Some(path) => self.list_history(Some(&path)),
            None => Vec::new(),
        }
    }

    pub fn read_snapshot(&self, snapshot_path: &str) -> Result<String, String> {
        fs::read_to_string(snapshot_path)
            .map_err(|error| format!("Could not read {snapshot_path}: {error}"))
    }

    pub fn diff_snapshot(&self, snapshot_path: &str) -> Result<SnapshotDiff, String> {
        let entries: Vec<HistoryEntry> = read_manifest(&self.history_manifest());
        let entry = entries
            .into_iter()
            .find(|entry| entry.snapshot_path == snapshot_path)
            .ok_or_else(|| "That snapshot is not in the history.".to_string())?;
        let snapshot = fs::read_to_string(&entry.snapshot_path)
            .map_err(|error| format!("Could not read the snapshot: {error}"))?;
        let current = fs::read_to_string(&entry.path).unwrap_or_default();
        let diff = TextDiff::from_lines(&snapshot, &current)
            .unified_diff()
            .header("snapshot", "current")
            .to_string();
        Ok(SnapshotDiff {
            path: entry.path,
            snapshot,
            current,
            diff,
        })
    }

    pub fn rollback(&self, snapshot_path: &str) -> Result<ScanResult, String> {
        let entries: Vec<HistoryEntry> = read_manifest(&self.history_manifest());
        let entry = entries
            .into_iter()
            .find(|entry| entry.snapshot_path == snapshot_path)
            .ok_or_else(|| "That snapshot is not in the history.".to_string())?;
        let contents = fs::read_to_string(&entry.snapshot_path)
            .map_err(|error| format!("Could not read the snapshot: {error}"))?;
        let path = PathBuf::from(&entry.path);
        let artifact = self
            .scan()
            .artifacts
            .into_iter()
            .find(|artifact| artifact.primary_path.eq_ignore_ascii_case(&entry.path))
            .unwrap_or_else(|| {
                Artifact::new(
                    entry.ecosystem.unwrap_or(Ecosystem::Claude),
                    ArtifactType::Settings,
                    Scope::Global,
                    entry.artifact_name.clone().unwrap_or_else(|| "file".into()),
                    entry.path.clone(),
                )
            });
        self.snapshot(&path, "rollback", &artifact)?;
        write_atomic(&path, &contents)?;
        Ok(self.scan())
    }

    pub fn prune_snapshots(&self, ids: &[String]) -> Result<usize, String> {
        let mut entries: Vec<HistoryEntry> = read_manifest(&self.history_manifest());
        let mut removed = 0;
        entries.retain(|entry| {
            if ids.contains(&entry.id) {
                let _ = fs::remove_file(&entry.snapshot_path);
                removed += 1;
                false
            } else {
                true
            }
        });
        write_manifest(&self.history_manifest(), &entries)?;
        Ok(removed)
    }
}

fn mcp_key_segments_error(artifact: &Artifact) -> Vec<String> {
    match artifact.ecosystem {
        Ecosystem::Opencode => {
            if artifact
                .key_path
                .as_deref()
                .is_some_and(|path| path.starts_with("mcp.servers."))
            {
                vec!["mcp".into(), "servers".into(), artifact.name.clone()]
            } else {
                vec!["mcp".into(), artifact.name.clone()]
            }
        }
        _ => vec!["mcpServers".into(), artifact.name.clone()],
    }
}

fn ecosystem_key(ecosystem: Ecosystem) -> &'static str {
    match ecosystem {
        Ecosystem::Claude => "claude",
        Ecosystem::Codex => "codex",
        Ecosystem::Opencode => "opencode",
        Ecosystem::CommandCode => "commandcode",
    }
}

fn scope_key(scope: Scope) -> &'static str {
    match scope {
        Scope::Global => "global",
        Scope::Project => "project",
        Scope::ProjectLocal => "project-local",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn manager(home: &Path, project: &Path) -> Manager {
        Manager::new(crate::roots::Roots::new(
            home.to_path_buf(),
            home.join("appdata"),
            Some(project.to_path_buf()),
        ))
    }

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn artifact_id(manager: &Manager, name: &str) -> String {
        manager
            .scan()
            .artifacts
            .into_iter()
            .find(|artifact| artifact.name == name)
            .unwrap_or_else(|| panic!("artifact {name} was not found"))
            .id
    }

    #[test]
    fn claude_agent_disable_keeps_comments_and_reverses_cleanly() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write(
            &project.join(".claude/agents/reviewer.md"),
            "---\nname: reviewer\ndescription: Reviews code\n---\nBody\n",
        );
        let settings = project.join(".claude/settings.local.json");
        write(&settings, "{\n  // keep this note\n  \"permissions\": { \"deny\": [] }\n}\n");

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "reviewer");

        let plan = manager.plan_toggle(&id, false).unwrap();
        manager.apply_plan(&plan).unwrap();
        let disabled = fs::read_to_string(&settings).unwrap();
        assert!(disabled.contains("keep this note"));
        assert!(disabled.contains("\"Agent(reviewer)\""));
        assert!(!manager.list_history(Some(&settings.to_string_lossy())).is_empty());

        let plan = manager.plan_toggle(&id, true).unwrap();
        manager.apply_plan(&plan).unwrap();
        let enabled = fs::read_to_string(&settings).unwrap();
        assert!(enabled.contains("keep this note"));
        assert!(!enabled.contains("Agent(reviewer)"));
    }

    #[test]
    fn codex_agents_move_to_quarantine_and_restore() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        let agent = home.path().join(".codex/agents/reviewer.toml");
        write(
            &agent,
            "name = \"reviewer\"\ndescription = \"Reviews\"\ndeveloper_instructions = \"Be strict\"\n",
        );

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "reviewer");
        let plan = manager.plan_toggle(&id, false).unwrap();
        manager.apply_plan(&plan).unwrap();

        assert!(!agent.exists());
        let entries = manager.list_trash();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, "quarantine");

        manager.restore(&entries[0].id).unwrap();
        assert!(agent.exists());
    }

    #[test]
    fn command_code_skills_use_the_disabled_skills_list() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write(
            &project.join(".commandcode/skills/reviewer/SKILL.md"),
            "---\nname: reviewer\ndescription: Reviews code\n---\nBody\n",
        );

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "reviewer");
        let plan = manager.plan_toggle(&id, false).unwrap();
        manager.apply_plan(&plan).unwrap();

        let settings = project.join(".commandcode/settings.json");
        let text = fs::read_to_string(&settings).unwrap();
        assert!(text.contains("disabledSkills"));
        assert!(text.contains("reviewer"));

        let plan = manager.plan_toggle(&id, true).unwrap();
        manager.apply_plan(&plan).unwrap();
        let text = fs::read_to_string(&settings).unwrap();
        assert!(!text.contains("reviewer"));
    }

    #[test]
    fn opencode_mcp_servers_toggle_the_disabled_flag() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let config = home.path().join(".config/opencode/opencode.json");
        write(
            &config,
            "{\n  // keep\n  \"mcp\": { \"servers\": { \"remote\": { \"type\": \"remote\", \"url\": \"https://x\" } } }\n}\n",
        );

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "remote");
        let plan = manager.plan_toggle(&id, false).unwrap();
        manager.apply_plan(&plan).unwrap();

        let text = fs::read_to_string(&config).unwrap();
        assert!(text.contains("keep"));
        assert!(text.contains("\"disabled\": true"));
        assert_eq!(
            manager
                .scan()
                .artifacts
                .into_iter()
                .find(|artifact| artifact.name == "remote")
                .unwrap()
                .enabled,
            Some(false)
        );

        let plan = manager.plan_toggle(&id, true).unwrap();
        manager.apply_plan(&plan).unwrap();
        assert!(!fs::read_to_string(&config).unwrap().contains("\"disabled\": true"));
    }

    #[test]
    fn remove_moves_a_file_to_trash_and_restore_returns_it() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        let agent = home.path().join(".claude/agents/reviewer.md");
        write(
            &agent,
            "---\nname: reviewer\ndescription: Reviews code\n---\nBody\n",
        );

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "reviewer");
        let plan = manager.plan_remove(&id).unwrap();
        manager.apply_plan(&plan).unwrap();

        assert!(!agent.exists());
        assert!(manager.scan().artifacts.is_empty());
        let entries = manager.list_trash();
        assert_eq!(entries.len(), 1);
        manager.restore(&entries[0].id).unwrap();
        assert!(agent.exists());
        assert_eq!(manager.scan().artifacts.len(), 1);
    }

    #[test]
    fn removing_a_config_key_is_restorable() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let config = home.path().join(".commandcode/mcp.json");
        let original = "{\n  \"mcpServers\": {\n    \"server\": { \"command\": \"node\" }\n  }\n}\n";
        write(&config, original);

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "server");
        let plan = manager.plan_remove(&id).unwrap();
        manager.apply_plan(&plan).unwrap();

        assert!(!fs::read_to_string(&config).unwrap().contains("\"server\""));
        let entry = manager.list_trash().pop().unwrap();
        assert_eq!(entry.kind, "configKey");
        manager.restore(&entry.id).unwrap();
        assert_eq!(fs::read_to_string(&config).unwrap(), original);
    }

    #[test]
    fn a_write_aborts_when_the_file_changed_since_the_scan() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        let agent = project.join(".claude/agents/reviewer.md");
        write(
            &agent,
            "---\nname: reviewer\ndescription: Reviews code\n---\nBody\n",
        );

        let manager = manager(home.path(), &project);
        let artifact = manager.scan().artifacts.pop().unwrap();
        write(
            &agent,
            "---\nname: reviewer\ndescription: Changed elsewhere\n---\nBody\n",
        );

        let error = manager
            .edit_file(
                &agent,
                &artifact,
                "disable",
                &HashMap::new(),
                "",
                |text| Ok(text.to_string()),
            )
            .unwrap_err();
        assert!(error.contains("changed since the last scan"));
    }

    #[test]
    fn history_rollback_restores_the_snapshot() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        let settings = project.join(".claude/settings.local.json");
        write(&settings, "{\n  \"permissions\": { \"deny\": [] }\n}\n");
        write(
            &project.join(".claude/agents/reviewer.md"),
            "---\nname: reviewer\ndescription: Reviews code\n---\nBody\n",
        );

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "reviewer");
        let plan = manager.plan_toggle(&id, false).unwrap();
        manager.apply_plan(&plan).unwrap();
        assert!(fs::read_to_string(&settings).unwrap().contains("Agent(reviewer)"));

        let entry = manager.list_history(Some(&settings.to_string_lossy())).pop().unwrap();
        manager.rollback(&entry.snapshot_path).unwrap();
        assert!(!fs::read_to_string(&settings).unwrap().contains("Agent(reviewer)"));
    }

    #[test]
    fn command_code_global_skills_write_the_global_settings() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        write(
            &home.path().join(".commandcode/skills/reviewer/SKILL.md"),
            "---\nname: reviewer\ndescription: Reviews code\n---\nBody\n",
        );

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "reviewer");
        let plan = manager.plan_toggle(&id, false).unwrap();
        manager.apply_plan(&plan).unwrap();

        assert!(home.path().join(".commandcode/settings.json").is_file());
        assert!(!project.join(".commandcode/settings.json").is_file());
    }

    #[test]
    fn claude_project_mcp_uses_the_project_disabled_list() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let mcp = project.join(".mcp.json");
        write(
            &mcp,
            "{\n  \"mcpServers\": { \"server\": { \"command\": \"node\" } }\n}\n",
        );
        let settings = project.join(".claude/settings.local.json");
        write(&settings, "{\n  // keep\n}\n");

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "server");
        let plan = manager.plan_toggle(&id, false).unwrap();
        manager.apply_plan(&plan).unwrap();

        let text = fs::read_to_string(&settings).unwrap();
        assert!(text.contains("keep"));
        assert!(text.contains("disabledMcpjsonServers"));
        assert!(!fs::read_to_string(&mcp).unwrap().contains("disabledMcpServers"));
    }

    #[test]
    fn removing_a_skill_moves_the_whole_directory() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        let directory = project.join(".claude/skills/reviewer");
        write(
            &directory.join("SKILL.md"),
            "---\nname: reviewer\ndescription: Reviews code\n---\nBody\n",
        );
        write(&directory.join("helper.txt"), "helper\n");

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "reviewer");
        let plan = manager.plan_remove(&id).unwrap();
        manager.apply_plan(&plan).unwrap();

        assert!(!directory.exists());
        let entry = manager.list_trash().pop().unwrap();
        manager.restore(&entry.id).unwrap();
        assert!(directory.join("helper.txt").is_file());
    }

    #[test]
    fn apply_aborts_when_another_writer_touched_the_target() {
        let home = tempdir().unwrap();
        let project = home.path().join("project");
        let settings = project.join(".claude/settings.local.json");
        write(&settings, "{\n  \"permissions\": { \"deny\": [] }\n}\n");
        write(
            &project.join(".claude/agents/reviewer.md"),
            "---\nname: reviewer\ndescription: Reviews code\n---\nBody\n",
        );

        let manager = manager(home.path(), &project);
        let id = artifact_id(&manager, "reviewer");
        let plan = manager.plan_toggle(&id, false).unwrap();
        write(&settings, "{\n  // changed elsewhere\n}\n");

        let error = manager.apply_plan(&plan).unwrap_err();
        assert!(error.contains("changed since the action was planned"));
    }

    #[test]
    fn action_payloads_round_trip_through_serde() {
        let plan = ActionPlan {
            action: ActionKind::Disable,
            artifact_id: "agent:global:x".into(),
            artifact_name: "x".into(),
            summary: "Disable x".into(),
            affected_ecosystems: vec![Ecosystem::Claude],
            files: vec![PlanFile {
                path: "C:/x".into(),
                operation: "update".into(),
                description: "edit".into(),
                expected_hash: Some("abc".into()),
            }],
            warnings: Vec::new(),
        };
        let json = serde_json::to_string(&plan).unwrap();
        let back: ActionPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(back.artifact_id, plan.artifact_id);
        assert_eq!(back.files[0].expected_hash.as_deref(), Some("abc"));

        let artifact = Artifact::new(
            Ecosystem::Claude,
            ArtifactType::Agent,
            Scope::Global,
            "x",
            "C:/x.md",
        );
        let json = serde_json::to_string(&artifact).unwrap();
        let back: Artifact = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "x");
        assert_eq!(back.artifact_type, ArtifactType::Agent);

        let entry = HistoryEntry {
            id: "1".into(),
            path: "C:/x".into(),
            action: "disable".into(),
            timestamp: "now".into(),
            ecosystem: Some(Ecosystem::Claude),
            artifact_name: Some("x".into()),
            snapshot_path: "C:/h".into(),
            hash: "abc".into(),
        };
        let back: HistoryEntry =
            serde_json::from_str(&serde_json::to_string(&entry).unwrap()).unwrap();
        assert_eq!(back.hash, "abc");

        let trash = TrashEntry {
            id: "1".into(),
            artifact_id: "a".into(),
            artifact_name: "x".into(),
            artifact_type: ArtifactType::Skill,
            ecosystem: Ecosystem::Codex,
            scope: Scope::Project,
            origin_path: "C:/x".into(),
            trashed_path: "C:/t".into(),
            timestamp: "now".into(),
            kind: "file".into(),
            restored: false,
            snapshot_path: None,
            after_hash: None,
        };
        let back: TrashEntry =
            serde_json::from_str(&serde_json::to_string(&trash).unwrap()).unwrap();
        assert_eq!(back.artifact_type, ArtifactType::Skill);

        let diff = SnapshotDiff {
            path: "C:/x".into(),
            snapshot: "a".into(),
            current: "b".into(),
            diff: "-a\n+b".into(),
        };
        let back: SnapshotDiff =
            serde_json::from_str(&serde_json::to_string(&diff).unwrap()).unwrap();
        assert_eq!(back.current, "b");

        let scan = ScanResult {
            artifacts: vec![Artifact::new(
                Ecosystem::Claude,
                ArtifactType::Agent,
                Scope::Global,
                "x",
                "C:/x.md",
            )],
            detections: vec![crate::model::EcosystemDetection {
                ecosystem: Ecosystem::Claude,
                detected: true,
                version: Some("2.1.215".into()),
                checked_paths: vec!["C:/x".into()],
                capabilities: vec!["disable-agent".into()],
                reason: None,
            }],
            warnings: Vec::new(),
            project_root: None,
            is_git_repository: false,
            scanned_at: "now".into(),
        };
        let back: ScanResult =
            serde_json::from_str(&serde_json::to_string(&scan).unwrap()).unwrap();
        assert_eq!(back.detections[0].capabilities, vec!["disable-agent"]);
    }
}

