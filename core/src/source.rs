use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::model::{
    Artifact, ArtifactType, Ecosystem, ReloadHint, Scope,
};
use crate::parser::parse_agent_markdown;
use crate::roots::{resolve_project_root, Roots};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    FilesRecursive,
    SkillDirs,
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
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();
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
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();
    format!("{:?}|{}|{}", artifact_type, normalized, key.unwrap_or(""))
}

fn is_allowed(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    let name = entry.file_name().to_string_lossy();
    !(name.starts_with('.') || name == "node_modules" || name == "target")
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

    if let Some(project) = &roots.project_root {
        let (git_root, is_git) = resolve_project_root(project);
        let mut directory = project.clone();
        let mut rank = 0u32;
        loop {
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
    }

    sources
}

pub fn sources(roots: &Roots) -> Vec<Source> {
    claude_agent_sources(roots)
}

pub fn collect(source: &Source, warnings: &mut Vec<String>) -> Vec<Candidate> {
    if !source.root.is_dir() {
        return Vec::new();
    }
    match source.layout {
        Layout::FilesRecursive => collect_files(source, warnings),
        Layout::SkillDirs => Vec::new(),
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
                warnings.push(format!("Could not read inside {}: {error}", source.root.display()));
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
        if source.format == Format::MarkdownAgent && extension != "md" {
            continue;
        }
        if source.format == Format::TomlAgent && extension != "toml" {
            continue;
        }
        match source.format {
            Format::MarkdownAgent => match read_agent(source, path) {
                Ok(candidate) => candidates.push(candidate),
                Err(warning) => warnings.push(warning),
            },
            _ => {}
        }
    }
    candidates
}

fn read_agent(source: &Source, path: &Path) -> Result<Candidate, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let fallback = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("unnamed")
        .to_string();
    let meta = parse_agent_markdown(&text, &fallback)
        .map_err(|error| format!("{}: {error}", path.display()))?;

    if let Some(reason) = &meta.skip_reason {
        return Err(format!(
            "{} was skipped: {reason}.",
            path.display()
        ));
    }

    let mut artifact = Artifact::new(
        source.ecosystem,
        source.artifact_type,
        source.scope,
        meta.name,
        path.to_string_lossy().to_string(),
    );
    artifact.description = meta.description;
    artifact.source_paths = vec![path.to_string_lossy().to_string()];
    artifact.shadowed = Vec::new();
    artifact.warnings = meta.warnings;
    artifact.can_disable = source.disable_mechanism.is_some();
    artifact.disable_mechanism = source.disable_mechanism.clone();
    artifact.reload = source.reload;
    artifact.ecosystems = vec![source.ecosystem];
    artifact.id = make_id(source.artifact_type, source.scope, path, None);

    Ok(Candidate {
        artifact,
        rank: source.rank,
        link: link_key(source.artifact_type, path, None),
    })
}
