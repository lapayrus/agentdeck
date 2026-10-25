use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::model::{Artifact, ArtifactType, Ecosystem, ReloadHint, Scope};
use crate::parser::{
    parse_agent_markdown, parse_agent_toml, parse_markdown, DescriptionRule,
};
use crate::roots::{resolve_project_root, Roots};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    FilesRecursive,
    SkillDirs,
    ConfigFile,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    MarkdownAgent,
    TomlAgent,
    SkillMarkdown,
    McpJson,
    McpToml,
    OtherFile,
}

#[derive(Clone, Debug)]
pub struct Source {
    pub ecosystem: Ecosystem,
    pub artifact_type: ArtifactType,
    pub scope: Scope,
    pub root: PathBuf,
    pub layout: Layout,
    pub format: Format,
    pub rank: u32,
    pub reload: ReloadHint,
    pub disable_mechanism: Option<String>,
}

pub struct Candidate {
    pub artifact: Artifact,
    pub rank: u32,
    pub link: String,
}

pub fn scope_key(scope: Scope) -> &'static str {
    match scope {
        Scope::Global => "global",
        Scope::Project => "project",
        Scope::ProjectLocal => "projectLocal",
    }
}

pub fn type_key(artifact_type: ArtifactType) -> &'static str {
    match artifact_type {
        ArtifactType::Agent => "agent",
        ArtifactType::Skill => "skill",
        ArtifactType::Mcp => "mcp",
        ArtifactType::Hook => "hook",
        ArtifactType::Command => "command",
        ArtifactType::Plugin => "plugin",
        ArtifactType::Memory => "memory",
        ArtifactType::Settings => "settings",
    }
}

pub fn make_id(
    artifact_type: ArtifactType,
    scope: Scope,
    path: &Path,
    key: Option<&str>,
) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    match key {
        Some(key) => format!(
            "{}:{}:{}#{}",
            type_key(artifact_type),
            scope_key(scope),
            normalized,
            key
        ),
        None => format!(
            "{}:{}:{}",
            type_key(artifact_type),
            scope_key(scope),
            normalized
        ),
    }
}

fn link_key(artifact_type: ArtifactType, path: &Path, key: Option<&str>) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    format!("{:?}|{}|{}", artifact_type, normalized, key.unwrap_or(""))
}

fn is_allowed(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    let name = entry.file_name().to_string_lossy();
    !(name.starts_with('.') || name == "node_modules" || name == "target")
}

fn project_layers(roots: &Roots) -> Vec<(PathBuf, u32)> {
    let mut layers = Vec::new();
    let Some(project) = &roots.project_root else {
        return layers;
    };
    let (git_root, is_git) = resolve_project_root(project);
    let mut directory = project.clone();
    let mut rank = 0u32;
    loop {
        layers.push((directory.clone(), rank));
        if !is_git || directory == git_root {
            break;
        }
        match directory.parent() {
            Some(parent) => {
                directory = parent.to_path_buf();
                rank += 1;
            }
            None => break,
        }
    }
    layers
}

pub fn claude_agent_sources(roots: &Roots) -> Vec<Source> {
    let mut sources = Vec::new();
    sources.push(Source {
        ecosystem: Ecosystem::Claude,
        artifact_type: ArtifactType::Agent,
        scope: Scope::Global,
        root: roots.home.join(".claude").join("agents"),
        layout: Layout::FilesRecursive,
        format: Format::MarkdownAgent,
        rank: 100,
        reload: ReloadHint::Live,
        disable_mechanism: Some("permissions.deny: [\"Agent(name)\"]".into()),
    });
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::Claude,
            artifact_type: ArtifactType::Agent,
            scope: Scope::Project,
            root: directory.join(".claude").join("agents"),
            layout: Layout::FilesRecursive,
            format: Format::MarkdownAgent,
            rank,
            reload: ReloadHint::Live,
            disable_mechanism: Some("permissions.deny: [\"Agent(name)\"]".into()),
        });
    }
    sources
}

pub fn codex_agent_sources(roots: &Roots) -> Vec<Source> {
    let mut sources = vec![Source {
        ecosystem: Ecosystem::Codex,
        artifact_type: ArtifactType::Agent,
        scope: Scope::Global,
        root: roots.home.join(".codex").join("agents"),
        layout: Layout::FilesRecursive,
        format: Format::TomlAgent,
        rank: 100,
        reload: ReloadHint::Restart,
        disable_mechanism: None,
    }];
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::Codex,
            artifact_type: ArtifactType::Agent,
            scope: Scope::Project,
            root: directory.join(".codex").join("agents"),
            layout: Layout::FilesRecursive,
            format: Format::TomlAgent,
            rank,
            reload: ReloadHint::Restart,
            disable_mechanism: None,
        });
    }
    sources
}

pub fn opencode_agent_sources(roots: &Roots) -> Vec<Source> {
    let global = roots.home.join(".config").join("opencode");
    let mut sources = vec![
        Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Agent,
            scope: Scope::Global,
            root: global.join("agents"),
            layout: Layout::FilesRecursive,
            format: Format::MarkdownAgent,
            rank: 100,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("disabled: true".into()),
        },
        Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Agent,
            scope: Scope::Global,
            root: global.join("agent"),
            layout: Layout::FilesRecursive,
            format: Format::MarkdownAgent,
            rank: 101,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("disabled: true".into()),
        },
    ];
    for (directory, rank) in project_layers(roots) {
        for (folder, offset) in [("agents", 0u32), ("agent", 1u32), ("modes", 2u32), ("mode", 2u32)] {
            sources.push(Source {
                ecosystem: Ecosystem::Opencode,
                artifact_type: ArtifactType::Agent,
                scope: Scope::Project,
                root: directory.join(".opencode").join(folder),
                layout: Layout::FilesRecursive,
                format: Format::MarkdownAgent,
                rank: rank * 10 + offset,
                reload: ReloadHint::NextSession,
                disable_mechanism: Some("disabled: true".into()),
            });
        }
    }
    sources
}

pub fn commandcode_agent_sources(roots: &Roots) -> Vec<Source> {
    let mut sources = vec![Source {
        ecosystem: Ecosystem::CommandCode,
        artifact_type: ArtifactType::Agent,
        scope: Scope::Global,
        root: roots.home.join(".commandcode").join("agents"),
        layout: Layout::FilesRecursive,
        format: Format::MarkdownAgent,
        rank: 0,
        reload: ReloadHint::NextTurn,
        disable_mechanism: None,
    }];
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Agent,
            scope: Scope::Project,
            root: directory.join(".commandcode").join("agents"),
            layout: Layout::FilesRecursive,
            format: Format::MarkdownAgent,
            rank: 100 + rank,
            reload: ReloadHint::NextTurn,
            disable_mechanism: None,
        });
    }
    sources
}

pub fn sources(roots: &Roots) -> Vec<Source> {
    let mut sources = Vec::new();
    sources.extend(claude_agent_sources(roots));
    sources.extend(codex_agent_sources(roots));
    sources.extend(opencode_agent_sources(roots));
    sources.extend(commandcode_agent_sources(roots));
    sources
}

pub fn collect(source: &Source, warnings: &mut Vec<String>) -> Vec<Candidate> {
    if !source.root.is_dir() {
        return Vec::new();
    }
    match source.layout {
        Layout::FilesRecursive => collect_files(source, warnings),
        Layout::SkillDirs => Vec::new(),
        Layout::ConfigFile => Vec::new(),
    }
}

fn collect_files(source: &Source, warnings: &mut Vec<String>) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for entry in WalkDir::new(&source.root)
        .max_depth(3)
        .into_iter()
        .filter_entry(is_allowed)
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warnings.push(format!(
                    "Could not read inside {}: {error}",
                    source.root.display()
                ));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let candidate = match source.format {
            Format::MarkdownAgent if extension == "md" => read_markdown_agent(source, path),
            Format::TomlAgent if extension == "toml" => read_toml_agent(source, path),
            _ => continue,
        };
        match candidate {
            Ok(candidate) => candidates.push(candidate),
            Err(warning) => warnings.push(warning),
        }
    }
    candidates
}

const RESERVED_COMMAND_CODE_AGENTS: [&str; 4] = ["explore", "plan", "review", "general"];

fn read_markdown_agent(source: &Source, path: &Path) -> Result<Candidate, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let fallback = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("unnamed")
        .to_string();
    let (require_name, description_rule, label) = match source.ecosystem {
        Ecosystem::Claude => (true, DescriptionRule::Warn, "Claude Code"),
        Ecosystem::Opencode => (false, DescriptionRule::Required, "OpenCode"),
        Ecosystem::CommandCode => (true, DescriptionRule::Optional, "Command Code"),
        Ecosystem::Codex => (true, DescriptionRule::Required, "Codex"),
    };
    let meta = parse_markdown(&text, &fallback, require_name, description_rule, label)
        .map_err(|error| format!("{}: {error}", path.display()))?;

    if let Some(reason) = &meta.skip_reason {
        return Err(format!("{} was skipped: {reason}.", path.display()));
    }
    if source.ecosystem == Ecosystem::CommandCode
        && RESERVED_COMMAND_CODE_AGENTS.contains(&meta.name.as_str())
    {
        return Err(format!(
            "{} was skipped: \"{}\" is a reserved Command Code agent name.",
            path.display(),
            meta.name
        ));
    }

    Ok(build_candidate(
        source,
        meta.name,
        meta.description,
        meta.warnings,
        path,
        None,
    ))
}

fn read_toml_agent(source: &Source, path: &Path) -> Result<Candidate, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let meta = parse_agent_toml(&text, path.to_string_lossy().as_ref())
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if let Some(reason) = &meta.skip_reason {
        return Err(format!("{} was skipped: {reason}.", path.display()));
    }
    Ok(build_candidate(
        source,
        meta.name,
        meta.description,
        meta.warnings,
        path,
        None,
    ))
}

pub fn build_candidate(
    source: &Source,
    name: String,
    description: Option<String>,
    warnings: Vec<String>,
    path: &Path,
    key: Option<&str>,
) -> Candidate {
    let mut artifact = Artifact::new(
        source.ecosystem,
        source.artifact_type,
        source.scope,
        name,
        path.to_string_lossy().to_string(),
    );
    artifact.description = description;
    artifact.source_paths = vec![path.to_string_lossy().to_string()];
    artifact.warnings = warnings;
    artifact.can_disable = source.disable_mechanism.is_some();
    artifact.disable_mechanism = source.disable_mechanism.clone();
    artifact.reload = source.reload;
    artifact.ecosystems = vec![source.ecosystem];
    artifact.id = make_id(source.artifact_type, source.scope, path, key);

    Candidate {
        artifact,
        rank: source.rank,
        link: link_key(source.artifact_type, path, key),
    }
}
