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
    SingleFile,
    EntryDirs,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    MarkdownAgent,
    TomlAgent,
    SkillMarkdown,
    McpJson,
    McpToml,
    CommandMarkdown,
    OtherFile,
    HookJson,
    PluginJson,
    PluginToml,
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

pub fn claude_skill_sources(roots: &Roots) -> Vec<Source> {
    let mut sources = vec![Source {
        ecosystem: Ecosystem::Claude,
        artifact_type: ArtifactType::Skill,
        scope: Scope::Global,
        root: roots.home.join(".claude").join("skills"),
        layout: Layout::SkillDirs,
        format: Format::SkillMarkdown,
        rank: 0,
        reload: ReloadHint::Live,
        disable_mechanism: Some("skillOverrides: {\"name\": \"off\"}".into()),
    }];
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::Claude,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Project,
            root: directory.join(".claude").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: 100 + rank,
            reload: ReloadHint::Live,
            disable_mechanism: Some("skillOverrides: {\"name\": \"off\"}".into()),
        });
    }
    sources
}

pub fn codex_skill_sources(roots: &Roots) -> Vec<Source> {
    let mut sources = vec![Source {
        ecosystem: Ecosystem::Codex,
        artifact_type: ArtifactType::Skill,
        scope: Scope::Global,
        root: roots.home.join(".agents").join("skills"),
        layout: Layout::SkillDirs,
        format: Format::SkillMarkdown,
        rank: 100,
        reload: ReloadHint::Restart,
        disable_mechanism: Some("[[skills.config]] enabled = false".into()),
    }];
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::Codex,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Project,
            root: directory.join(".agents").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank,
            reload: ReloadHint::Restart,
            disable_mechanism: Some("[[skills.config]] enabled = false".into()),
        });
    }
    sources
}

pub fn opencode_skill_sources(roots: &Roots) -> Vec<Source> {
    let global = roots.home.join(".config").join("opencode");
    let mut sources = vec![
        Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Global,
            root: global.join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: 20,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("permission: skill deny".into()),
        },
        Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Global,
            root: roots.home.join(".agents").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: 30,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("permission: skill deny".into()),
        },
        Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Global,
            root: roots.home.join(".claude").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: 40,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("permission: skill deny".into()),
        },
    ];
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Project,
            root: directory.join(".opencode").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: rank * 10,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("permission: skill deny".into()),
        });
        sources.push(Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Project,
            root: directory.join(".opencode").join("skill"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: rank * 10 + 1,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("permission: skill deny".into()),
        });
        for compat in [".agents", ".claude"] {
            sources.push(Source {
                ecosystem: Ecosystem::Opencode,
                artifact_type: ArtifactType::Skill,
                scope: Scope::Project,
                root: directory.join(compat).join("skills"),
                layout: Layout::SkillDirs,
                format: Format::SkillMarkdown,
                rank: rank * 10 + 10,
                reload: ReloadHint::NextSession,
                disable_mechanism: Some("permission: skill deny".into()),
            });
        }
    }
    sources
}

pub fn commandcode_skill_sources(roots: &Roots) -> Vec<Source> {
    let mut sources = vec![
        Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Global,
            root: roots.home.join(".commandcode").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: 20,
            reload: ReloadHint::NextTurn,
            disable_mechanism: Some("disabledSkills: [name]".into()),
        },
        Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Global,
            root: roots.home.join(".agents").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: 30,
            reload: ReloadHint::NextTurn,
            disable_mechanism: Some("disabledSkills: [name]".into()),
        },
    ];
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Project,
            root: directory.join(".commandcode").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: rank * 10,
            reload: ReloadHint::NextTurn,
            disable_mechanism: Some("disabledSkills: [name]".into()),
        });
        sources.push(Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Skill,
            scope: Scope::Project,
            root: directory.join(".agents").join("skills"),
            layout: Layout::SkillDirs,
            format: Format::SkillMarkdown,
            rank: rank * 10 + 10,
            reload: ReloadHint::NextTurn,
            disable_mechanism: Some("disabledSkills: [name]".into()),
        });
    }
    sources
}

pub fn mcp_sources(roots: &Roots) -> Vec<Source> {
    let mut sources = Vec::new();
    let claude_config = roots.home.join(".claude.json");
    sources.push(Source {
        ecosystem: Ecosystem::Claude,
        artifact_type: ArtifactType::Mcp,
        scope: Scope::Global,
        root: claude_config.clone(),
        layout: Layout::ConfigFile,
        format: Format::McpJson,
        rank: 20,
        reload: ReloadHint::NextSession,
        disable_mechanism: Some("disabledMcpServers".into()),
    });
    sources.push(Source {
        ecosystem: Ecosystem::Claude,
        artifact_type: ArtifactType::Mcp,
        scope: Scope::ProjectLocal,
        root: claude_config,
        layout: Layout::ConfigFile,
        format: Format::McpJson,
        rank: 0,
        reload: ReloadHint::NextSession,
        disable_mechanism: Some("disabledMcpServers".into()),
    });
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::Claude,
            artifact_type: ArtifactType::Mcp,
            scope: Scope::Project,
            root: directory.join(".mcp.json"),
            layout: Layout::ConfigFile,
            format: Format::McpJson,
            rank: 10 + rank * 10,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("disabledMcpjsonServers".into()),
        });
    }

    sources.push(Source {
        ecosystem: Ecosystem::Codex,
        artifact_type: ArtifactType::Mcp,
        scope: Scope::Global,
        root: roots.home.join(".codex").join("config.toml"),
        layout: Layout::ConfigFile,
        format: Format::McpToml,
        rank: 100,
        reload: ReloadHint::Restart,
        disable_mechanism: Some("enabled = false".into()),
    });
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::Codex,
            artifact_type: ArtifactType::Mcp,
            scope: Scope::Project,
            root: directory.join(".codex").join("config.toml"),
            layout: Layout::ConfigFile,
            format: Format::McpToml,
            rank: rank * 10,
            reload: ReloadHint::Restart,
            disable_mechanism: Some("enabled = false".into()),
        });
    }

    let opencode = roots.home.join(".config").join("opencode");
    for (name, rank) in [("opencode.json", 20u32), ("opencode.jsonc", 21u32)] {
        sources.push(Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Mcp,
            scope: Scope::Global,
            root: opencode.join(name),
            layout: Layout::ConfigFile,
            format: Format::McpJson,
            rank,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("disabled: true".into()),
        });
    }
    for (directory, rank) in project_layers(roots) {
        for (name, offset) in [
            ("opencode.json", 10u32),
            ("opencode.jsonc", 11u32),
        ] {
            sources.push(Source {
                ecosystem: Ecosystem::Opencode,
                artifact_type: ArtifactType::Mcp,
                scope: Scope::Project,
                root: directory.join(name),
                layout: Layout::ConfigFile,
                format: Format::McpJson,
                rank: rank * 100 + offset,
                reload: ReloadHint::NextSession,
                disable_mechanism: Some("disabled: true".into()),
            });
        }
        for (name, offset) in [
            ("opencode.json", 0u32),
            ("opencode.jsonc", 1u32),
        ] {
            sources.push(Source {
                ecosystem: Ecosystem::Opencode,
                artifact_type: ArtifactType::Mcp,
                scope: Scope::Project,
                root: directory.join(".opencode").join(name),
                layout: Layout::ConfigFile,
                format: Format::McpJson,
                rank: rank * 100 + offset,
                reload: ReloadHint::NextSession,
                disable_mechanism: Some("disabled: true".into()),
            });
        }
    }

    sources.push(Source {
        ecosystem: Ecosystem::CommandCode,
        artifact_type: ArtifactType::Mcp,
        scope: Scope::Global,
        root: roots.home.join(".commandcode").join("mcp.json"),
        layout: Layout::ConfigFile,
        format: Format::McpJson,
        rank: 20,
        reload: ReloadHint::NextSession,
        disable_mechanism: Some("enabled: false".into()),
    });
    for (directory, rank) in project_layers(roots) {
        sources.push(Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Mcp,
            scope: Scope::Project,
            root: directory.join(".mcp.json"),
            layout: Layout::ConfigFile,
            format: Format::McpJson,
            rank: 10 + rank * 10,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("enabled: false".into()),
        });
    }
    if let Some(project) = &roots.project_root {
        sources.push(Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Mcp,
            scope: Scope::ProjectLocal,
            root: roots
                .home
                .join(".commandcode")
                .join("projects")
                .join(project_slug(project))
                .join("mcp.json"),
            layout: Layout::ConfigFile,
            format: Format::McpJson,
            rank: 0,
            reload: ReloadHint::NextSession,
            disable_mechanism: Some("enabled: false".into()),
        });
    }

    sources
}

pub fn project_slug(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let mut expanded = String::with_capacity(raw.len() + 8);
    let mut previous_lower = false;
    for character in raw.chars() {
        if character.is_ascii_uppercase() && previous_lower {
            expanded.push('-');
        }
        previous_lower = character.is_ascii_lowercase() || character.is_ascii_digit();
        expanded.push(character);
    }
    let mut slug = String::new();
    let mut pending_dash = false;
    for character in expanded.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(character);
        } else {
            pending_dash = true;
        }
    }
    slug
}

pub fn read_only_sources(roots: &Roots) -> Vec<Source> {
    let mut sources = Vec::new();
    let global = |ecosystem: Ecosystem,
                  artifact_type: ArtifactType,
                  path: PathBuf,
                  format: Format,
                  rank: u32,
                  reload: ReloadHint| Source {
        ecosystem,
        artifact_type,
        scope: Scope::Global,
        root: path,
        layout: match format {
            Format::OtherFile => Layout::SingleFile,
            _ => Layout::ConfigFile,
        },
        format,
        rank,
        reload,
        disable_mechanism: None,
    };

    sources.push(global(
        Ecosystem::Claude,
        ArtifactType::Settings,
        roots.home.join(".claude").join("settings.json"),
        Format::OtherFile,
        100,
        ReloadHint::Live,
    ));
    sources.push(global(
        Ecosystem::Claude,
        ArtifactType::Hook,
        roots.home.join(".claude").join("settings.json"),
        Format::HookJson,
        100,
        ReloadHint::Live,
    ));
    sources.push(global(
        Ecosystem::Claude,
        ArtifactType::Memory,
        roots.home.join(".claude").join("CLAUDE.md"),
        Format::OtherFile,
        100,
        ReloadHint::Live,
    ));
    sources.push(Source {
        ecosystem: Ecosystem::Claude,
        artifact_type: ArtifactType::Plugin,
        scope: Scope::Global,
        root: roots
            .home
            .join(".claude")
            .join("plugins")
            .join("installed_plugins.json"),
        layout: Layout::ConfigFile,
        format: Format::PluginJson,
        rank: 100,
        reload: ReloadHint::Live,
        disable_mechanism: None,
    });
    sources.push(Source {
        ecosystem: Ecosystem::Claude,
        artifact_type: ArtifactType::Command,
        scope: Scope::Global,
        root: roots.home.join(".claude").join("commands"),
        layout: Layout::FilesRecursive,
        format: Format::CommandMarkdown,
        rank: 100,
        reload: ReloadHint::Live,
        disable_mechanism: None,
    });

    sources.push(global(
        Ecosystem::Codex,
        ArtifactType::Settings,
        roots.home.join(".codex").join("config.toml"),
        Format::OtherFile,
        100,
        ReloadHint::Restart,
    ));
    sources.push(Source {
        ecosystem: Ecosystem::Codex,
        artifact_type: ArtifactType::Plugin,
        scope: Scope::Global,
        root: roots.home.join(".codex").join("config.toml"),
        layout: Layout::ConfigFile,
        format: Format::PluginToml,
        rank: 101,
        reload: ReloadHint::Restart,
        disable_mechanism: None,
    });
    sources.push(global(
        Ecosystem::Codex,
        ArtifactType::Hook,
        roots.home.join(".codex").join("hooks.json"),
        Format::HookJson,
        100,
        ReloadHint::Restart,
    ));
    sources.push(global(
        Ecosystem::Codex,
        ArtifactType::Memory,
        roots.home.join(".codex").join("AGENTS.md"),
        Format::OtherFile,
        100,
        ReloadHint::NextTurn,
    ));

    sources.push(global(
        Ecosystem::Opencode,
        ArtifactType::Settings,
        roots.home.join(".config").join("opencode").join("opencode.json"),
        Format::OtherFile,
        100,
        ReloadHint::NextSession,
    ));
    sources.push(global(
        Ecosystem::Opencode,
        ArtifactType::Memory,
        roots.home.join(".config").join("opencode").join("AGENTS.md"),
        Format::OtherFile,
        100,
        ReloadHint::NextTurn,
    ));

    sources.push(global(
        Ecosystem::CommandCode,
        ArtifactType::Settings,
        roots.home.join(".commandcode").join("settings.json"),
        Format::OtherFile,
        100,
        ReloadHint::NextTurn,
    ));
    sources.push(Source {
        ecosystem: Ecosystem::CommandCode,
        artifact_type: ArtifactType::Command,
        scope: Scope::Global,
        root: roots.home.join(".commandcode").join("commands"),
        layout: Layout::FilesRecursive,
        format: Format::CommandMarkdown,
        rank: 100,
        reload: ReloadHint::NextTurn,
        disable_mechanism: None,
    });
    sources.push(Source {
        ecosystem: Ecosystem::CommandCode,
        artifact_type: ArtifactType::Plugin,
        scope: Scope::Global,
        root: roots.home.join(".commandcode").join("mods"),
        layout: Layout::EntryDirs,
        format: Format::OtherFile,
        rank: 100,
        reload: ReloadHint::NextTurn,
        disable_mechanism: None,
    });

    for (directory, rank) in project_layers(roots) {
        let settings_rank = 100 + rank * 10;
        for name in ["settings.json", "settings.local.json"] {
            sources.push(Source {
                ecosystem: Ecosystem::Claude,
                artifact_type: ArtifactType::Settings,
                scope: Scope::Project,
                root: directory.join(".claude").join(name),
                layout: Layout::SingleFile,
                format: Format::OtherFile,
                rank: settings_rank,
                reload: ReloadHint::Live,
                disable_mechanism: None,
            });
        }
        sources.push(Source {
            ecosystem: Ecosystem::Claude,
            artifact_type: ArtifactType::Hook,
            scope: Scope::Project,
            root: directory.join(".claude").join("settings.json"),
            layout: Layout::ConfigFile,
            format: Format::HookJson,
            rank: settings_rank,
            reload: ReloadHint::Live,
            disable_mechanism: None,
        });
        for name in ["CLAUDE.md", "AGENTS.md"] {
            sources.push(Source {
                ecosystem: Ecosystem::Claude,
                artifact_type: ArtifactType::Memory,
                scope: Scope::Project,
                root: directory.join(name),
                layout: Layout::SingleFile,
                format: Format::OtherFile,
                rank: settings_rank,
                reload: ReloadHint::Live,
                disable_mechanism: None,
            });
        }
        sources.push(Source {
            ecosystem: Ecosystem::Claude,
            artifact_type: ArtifactType::Command,
            scope: Scope::Project,
            root: directory.join(".claude").join("commands"),
            layout: Layout::FilesRecursive,
            format: Format::CommandMarkdown,
            rank: settings_rank,
            reload: ReloadHint::Live,
            disable_mechanism: None,
        });

        sources.push(Source {
            ecosystem: Ecosystem::Codex,
            artifact_type: ArtifactType::Settings,
            scope: Scope::Project,
            root: directory.join(".codex").join("config.toml"),
            layout: Layout::SingleFile,
            format: Format::OtherFile,
            rank: settings_rank,
            reload: ReloadHint::Restart,
            disable_mechanism: None,
        });
        sources.push(Source {
            ecosystem: Ecosystem::Codex,
            artifact_type: ArtifactType::Plugin,
            scope: Scope::Project,
            root: directory.join(".codex").join("config.toml"),
            layout: Layout::ConfigFile,
            format: Format::PluginToml,
            rank: settings_rank,
            reload: ReloadHint::Restart,
            disable_mechanism: None,
        });
        sources.push(Source {
            ecosystem: Ecosystem::Codex,
            artifact_type: ArtifactType::Hook,
            scope: Scope::Project,
            root: directory.join(".codex").join("hooks.json"),
            layout: Layout::ConfigFile,
            format: Format::HookJson,
            rank: settings_rank,
            reload: ReloadHint::Restart,
            disable_mechanism: None,
        });
        for name in ["AGENTS.md", "AGENTS.override.md"] {
            sources.push(Source {
                ecosystem: Ecosystem::Codex,
                artifact_type: ArtifactType::Memory,
                scope: Scope::Project,
                root: directory.join(name),
                layout: Layout::SingleFile,
                format: Format::OtherFile,
                rank: settings_rank,
                reload: ReloadHint::NextTurn,
                disable_mechanism: None,
            });
        }

        for name in ["opencode.json", "opencode.jsonc"] {
            sources.push(Source {
                ecosystem: Ecosystem::Opencode,
                artifact_type: ArtifactType::Settings,
                scope: Scope::Project,
                root: directory.join(name),
                layout: Layout::SingleFile,
                format: Format::OtherFile,
                rank: settings_rank,
                reload: ReloadHint::NextSession,
                disable_mechanism: None,
            });
        }
        sources.push(Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Command,
            scope: Scope::Project,
            root: directory.join(".opencode").join("commands"),
            layout: Layout::FilesRecursive,
            format: Format::CommandMarkdown,
            rank: settings_rank,
            reload: ReloadHint::NextSession,
            disable_mechanism: None,
        });
        sources.push(Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Plugin,
            scope: Scope::Project,
            root: directory.join(".opencode").join("plugins"),
            layout: Layout::EntryDirs,
            format: Format::OtherFile,
            rank: settings_rank,
            reload: ReloadHint::NextSession,
            disable_mechanism: None,
        });
        sources.push(Source {
            ecosystem: Ecosystem::Opencode,
            artifact_type: ArtifactType::Memory,
            scope: Scope::Project,
            root: directory.join("AGENTS.md"),
            layout: Layout::SingleFile,
            format: Format::OtherFile,
            rank: settings_rank,
            reload: ReloadHint::NextTurn,
            disable_mechanism: None,
        });

        for name in ["settings.json", "settings.local.json"] {
            sources.push(Source {
                ecosystem: Ecosystem::CommandCode,
                artifact_type: ArtifactType::Settings,
                scope: Scope::Project,
                root: directory.join(".commandcode").join(name),
                layout: Layout::SingleFile,
                format: Format::OtherFile,
                rank: settings_rank,
                reload: ReloadHint::NextTurn,
                disable_mechanism: None,
            });
        }
        sources.push(Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Command,
            scope: Scope::Project,
            root: directory.join(".commandcode").join("commands"),
            layout: Layout::FilesRecursive,
            format: Format::CommandMarkdown,
            rank: settings_rank,
            reload: ReloadHint::NextTurn,
            disable_mechanism: None,
        });
        sources.push(Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Plugin,
            scope: Scope::Project,
            root: directory.join(".commandcode").join("mods"),
            layout: Layout::EntryDirs,
            format: Format::OtherFile,
            rank: settings_rank,
            reload: ReloadHint::NextTurn,
            disable_mechanism: None,
        });
        sources.push(Source {
            ecosystem: Ecosystem::CommandCode,
            artifact_type: ArtifactType::Memory,
            scope: Scope::Project,
            root: directory.join("AGENTS.md"),
            layout: Layout::SingleFile,
            format: Format::OtherFile,
            rank: settings_rank,
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
    sources.extend(claude_skill_sources(roots));
    sources.extend(codex_skill_sources(roots));
    sources.extend(opencode_skill_sources(roots));
    sources.extend(commandcode_skill_sources(roots));
    sources.extend(mcp_sources(roots));
    sources.extend(read_only_sources(roots));
    sources
}

pub fn collect(
    source: &Source,
    project_root: Option<&Path>,
    warnings: &mut Vec<String>,
) -> Vec<Candidate> {
    if source.layout == Layout::ConfigFile {
        if !source.root.is_file() {
            return Vec::new();
        }
        return collect_config(source, project_root, warnings);
    }
    if source.layout == Layout::SingleFile {
        return collect_single_file(source);
    }
    if !source.root.is_dir() {
        return Vec::new();
    }
    match source.layout {
        Layout::FilesRecursive => collect_files(source, warnings),
        Layout::SkillDirs => collect_skill_dirs(source, warnings),
        Layout::EntryDirs => collect_entry_dirs(source, warnings),
        Layout::SingleFile | Layout::ConfigFile => Vec::new(),
    }
}

fn collect_single_file(source: &Source) -> Vec<Candidate> {
    if !source.root.is_file() {
        return Vec::new();
    }
    let name = source
        .root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("file")
        .to_string();
    vec![build_candidate(
        source,
        name,
        None,
        Vec::new(),
        &source.root,
        None,
    )]
}

fn collect_entry_dirs(source: &Source, warnings: &mut Vec<String>) -> Vec<Candidate> {
    let entries = match fs::read_dir(&source.root) {
        Ok(entries) => entries,
        Err(error) => {
            warnings.push(format!(
                "Could not read {}: {error}",
                source.root.display()
            ));
            return Vec::new();
        }
    };
    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        candidates.push(build_candidate(
            source,
            name,
            None,
            Vec::new(),
            &entry.path(),
            None,
        ));
    }
    candidates
}

fn collect_config(
    source: &Source,
    project_root: Option<&Path>,
    warnings: &mut Vec<String>,
) -> Vec<Candidate> {
    let text = match fs::read_to_string(&source.root) {
        Ok(text) => text,
        Err(error) => {
            warnings.push(format!(
                "Could not read {}: {error}",
                source.root.display()
            ));
            return Vec::new();
        }
    };
    match source.format {
        Format::McpJson => match crate::jsonc::parse(&text) {
            Ok(value) => json_mcp_candidates(source, project_root, &value, warnings),
            Err(error) => {
                warnings.push(format!("{}: {error}", source.root.display()));
                Vec::new()
            }
        },
        Format::McpToml => toml_mcp_candidates(source, &text, warnings),
        Format::HookJson => hook_json_candidates(source, &text, warnings),
        Format::PluginJson => plugin_json_candidates(source, &text, warnings),
        Format::PluginToml => plugin_toml_candidates(source, &text, warnings),
        _ => Vec::new(),
    }
}

fn string_list(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|text| text.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn normalize_path(text: &str) -> String {
    let text = text.replace('\\', "/");
    let text = text.strip_prefix("//?/").unwrap_or(&text);
    text.trim_end_matches('/').to_lowercase()
}

fn find_project<'a>(
    value: &'a serde_json::Value,
    project_root: &Path,
) -> Option<&'a serde_json::Value> {
    let projects = value.get("projects")?.as_object()?;
    let target = normalize_path(&project_root.to_string_lossy());
    projects
        .iter()
        .find(|(key, _)| normalize_path(key) == target)
        .map(|(_, entry)| entry)
}

fn json_command(object: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    match object.get("command") {
        Some(serde_json::Value::String(text)) => parts.push(text.clone()),
        Some(serde_json::Value::Array(items)) => {
            parts.extend(items.iter().filter_map(|item| {
                item.as_str().map(|text| text.to_string())
            }));
        }
        _ => {}
    }
    if !matches!(object.get("command"), Some(serde_json::Value::Array(_))) {
        if let Some(args) = object.get("args").and_then(|value| value.as_array()) {
            parts.extend(
                args.iter()
                    .filter_map(|item| item.as_str().map(|text| text.to_string())),
            );
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn infer_transport(
    declared: Option<&str>,
    url: Option<&str>,
    command: Option<&str>,
) -> String {
    match declared {
        Some("local") => "stdio".into(),
        Some("remote") => "http".into(),
        Some(other) => other.to_string(),
        None if url.is_some() => "http".into(),
        None if command.is_some() => "stdio".into(),
        None => "unknown".into(),
    }
}

fn json_mcp_candidates(
    source: &Source,
    project_root: Option<&Path>,
    value: &serde_json::Value,
    warnings: &mut Vec<String>,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut disabled: Vec<String> = Vec::new();
    let (map, prefix) = match source.ecosystem {
        Ecosystem::Claude => {
            if source.scope == Scope::ProjectLocal {
                let Some(root) = project_root else {
                    return candidates;
                };
                let Some(project) = find_project(value, root) else {
                    return candidates;
                };
                disabled = string_list(project.get("disabledMcpServers"));
                (
                    project.get("mcpServers").and_then(|value| value.as_object()),
                    format!("projects[{}].mcpServers", root.display()),
                )
            } else {
                (
                    value.get("mcpServers").and_then(|value| value.as_object()),
                    "mcpServers".to_string(),
                )
            }
        }
        Ecosystem::Opencode => {
            let mcp = value.get("mcp").and_then(|value| value.as_object());
            match mcp {
                Some(table) if table.get("servers").is_some() => (
                    table.get("servers").and_then(|value| value.as_object()),
                    "mcp.servers".to_string(),
                ),
                Some(table) => (Some(table), "mcp".to_string()),
                None => (None, "mcp".to_string()),
            }
        }
        _ => (
            value.get("mcpServers").and_then(|value| value.as_object()),
            "mcpServers".to_string(),
        ),
    };

    let Some(map) = map else {
        return candidates;
    };
    for (name, entry) in map {
        let Some(object) = entry.as_object() else {
            warnings.push(format!(
                "{}: the MCP entry \"{name}\" is not an object and was skipped.",
                source.root.display()
            ));
            continue;
        };
        let command = json_command(object);
        let url = object
            .get("url")
            .and_then(|value| value.as_str())
            .map(|text| text.to_string());
        let declared = object
            .get("type")
            .or_else(|| object.get("transport"))
            .and_then(|value| value.as_str());
        let transport = infer_transport(declared, url.as_deref(), command.as_deref());
        let enabled = !disabled.iter().any(|entry| entry == name)
            && object.get("disabled").and_then(|value| value.as_bool()) != Some(true)
            && object.get("enabled").and_then(|value| value.as_bool()) != Some(false);

        let key_path = format!("{prefix}.{name}");
        let mut candidate = build_candidate(
            source,
            name.clone(),
            None,
            Vec::new(),
            &source.root,
            Some(&key_path),
        );
        candidate.artifact.key_path = Some(key_path);
        candidate.artifact.transport = Some(transport);
        candidate.artifact.command = command;
        candidate.artifact.url = url;
        candidate.artifact.enabled = Some(enabled);
        candidates.push(candidate);
    }
    candidates
}

fn toml_mcp_candidates(
    source: &Source,
    text: &str,
    warnings: &mut Vec<String>,
) -> Vec<Candidate> {
    let document = match text.parse::<toml_edit::DocumentMut>() {
        Ok(document) => document,
        Err(error) => {
            warnings.push(format!(
                "{}: invalid TOML: {error}",
                source.root.display()
            ));
            return Vec::new();
        }
    };
    let Some(table) = document
        .get("mcp_servers")
        .and_then(|item| item.as_table_like())
    else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for (name, item) in table.iter() {
        let Some(entry) = item.as_table_like() else {
            warnings.push(format!(
                "{}: the MCP entry \"{name}\" is not a table and was skipped.",
                source.root.display()
            ));
            continue;
        };
        let mut parts: Vec<String> = Vec::new();
        if let Some(command) = entry.get("command").and_then(|item| item.as_str()) {
            parts.push(command.to_string());
        }
        if let Some(args) = entry.get("args").and_then(|item| item.as_array()) {
            parts.extend(
                args.iter()
                    .filter_map(|value| value.as_str().map(|text| text.to_string())),
            );
        }
        let command = if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        };
        let url = entry
            .get("url")
            .and_then(|item| item.as_str())
            .map(|text| text.to_string());
        let declared = entry
            .get("type")
            .or_else(|| entry.get("transport"))
            .and_then(|item| item.as_str());
        let transport = infer_transport(declared, url.as_deref(), command.as_deref());
        let enabled = entry
            .get("enabled")
            .and_then(|item| item.as_bool())
            .unwrap_or(true);
        let key_path = format!("mcp_servers.{name}");
        let mut candidate = build_candidate(
            source,
            name.to_string(),
            None,
            Vec::new(),
            &source.root,
            Some(&key_path),
        );
        candidate.artifact.key_path = Some(key_path);
        candidate.artifact.transport = Some(transport);
        candidate.artifact.command = command;
        candidate.artifact.url = url;
        candidate.artifact.enabled = Some(enabled);
        candidates.push(candidate);
    }
    candidates
}

fn hook_json_candidates(
    source: &Source,
    text: &str,
    warnings: &mut Vec<String>,
) -> Vec<Candidate> {
    let value = match crate::jsonc::parse(text) {
        Ok(value) => value,
        Err(error) => {
            warnings.push(format!("{}: {error}", source.root.display()));
            return Vec::new();
        }
    };
    let Some(hooks) = value.get("hooks").and_then(|value| value.as_object()) else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    for (event, entry) in hooks {
        let key_path = format!("hooks.{event}");
        let description = entry
            .as_array()
            .map(|handlers| format!("{} handler(s)", handlers.len()));
        let mut candidate = build_candidate(
            source,
            event.clone(),
            description,
            Vec::new(),
            &source.root,
            Some(&key_path),
        );
        candidate.artifact.key_path = Some(key_path);
        candidates.push(candidate);
    }
    candidates
}

fn plugin_json_candidates(
    source: &Source,
    text: &str,
    warnings: &mut Vec<String>,
) -> Vec<Candidate> {
    let value = match crate::jsonc::parse(text) {
        Ok(value) => value,
        Err(error) => {
            warnings.push(format!("{}: {error}", source.root.display()));
            return Vec::new();
        }
    };
    let Some(plugins) = value.get("plugins").and_then(|value| value.as_object()) else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    for name in plugins.keys() {
        let key_path = format!("plugins.{name}");
        let mut candidate = build_candidate(
            source,
            name.clone(),
            None,
            Vec::new(),
            &source.root,
            Some(&key_path),
        );
        candidate.artifact.key_path = Some(key_path);
        candidates.push(candidate);
    }
    candidates
}

fn plugin_toml_candidates(
    source: &Source,
    text: &str,
    warnings: &mut Vec<String>,
) -> Vec<Candidate> {
    let document = match text.parse::<toml_edit::DocumentMut>() {
        Ok(document) => document,
        Err(error) => {
            warnings.push(format!(
                "{}: invalid TOML: {error}",
                source.root.display()
            ));
            return Vec::new();
        }
    };
    let Some(plugins) = document.get("plugins").and_then(|item| item.as_table_like()) else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    for (name, _) in plugins.iter() {
        let key_path = format!("plugins.{name}");
        let mut candidate = build_candidate(
            source,
            name.to_string(),
            None,
            Vec::new(),
            &source.root,
            Some(&key_path),
        );
        candidate.artifact.key_path = Some(key_path);
        candidates.push(candidate);
    }
    candidates
}

fn read_command(source: &Source, path: &Path) -> Result<Candidate, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("command")
        .to_string();
    let description = match crate::parser::split_frontmatter(&text) {
        Ok(Some(frontmatter)) => {
            crate::parser::string_field(&frontmatter.data, "description")
        }
        Ok(None) => None,
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    Ok(build_candidate(
        source,
        name,
        description,
        Vec::new(),
        path,
        None,
    ))
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
            Format::CommandMarkdown if extension == "md" => read_command(source, path),
            _ => continue,
        };
        match candidate {
            Ok(candidate) => candidates.push(candidate),
            Err(warning) => warnings.push(warning),
        }
    }
    candidates
}

fn collect_skill_dirs(source: &Source, warnings: &mut Vec<String>) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let entries = match fs::read_dir(&source.root) {
        Ok(entries) => entries,
        Err(error) => {
            warnings.push(format!(
                "Could not read {}: {error}",
                source.root.display()
            ));
            return candidates;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        if folder.starts_with('.') || folder == "node_modules" {
            continue;
        }
        let skill_file = path.join("SKILL.md");
        if !skill_file.is_file() {
            continue;
        }
        match read_skill(source, &path, &skill_file) {
            Ok(candidate) => candidates.push(candidate),
            Err(warning) => warnings.push(warning),
        }
    }
    candidates
}

fn read_skill(source: &Source, directory: &Path, skill_file: &Path) -> Result<Candidate, String> {
    let text = fs::read_to_string(skill_file)
        .map_err(|error| format!("Could not read {}: {error}", skill_file.display()))?;
    let fallback = directory
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unnamed")
        .to_string();
    let (require_name, description_rule, label) = match source.ecosystem {
        Ecosystem::Claude => (false, DescriptionRule::Warn, "Claude Code"),
        Ecosystem::Codex => (true, DescriptionRule::Required, "Codex"),
        Ecosystem::Opencode => (false, DescriptionRule::Required, "OpenCode"),
        Ecosystem::CommandCode => (true, DescriptionRule::Required, "Command Code"),
    };
    let meta = parse_markdown(&text, &fallback, require_name, description_rule, label)
        .map_err(|error| format!("{}: {error}", skill_file.display()))?;
    if let Some(reason) = &meta.skip_reason {
        return Err(format!("{} was skipped: {reason}.", skill_file.display()));
    }

    let mut warnings = meta.warnings;
    if source.ecosystem == Ecosystem::CommandCode {
        let valid = meta.name.len() <= 64
            && meta
                .name
                .chars()
                .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-');
        if !valid {
            warnings.push(
                "Command Code requires a skill name of lowercase letters, digits and hyphens.".into(),
            );
        }
    }
    if let Some(description) = &meta.description {
        if description.len() > 1024 {
            warnings.push("The description is longer than 1024 characters.".into());
        }
    }

    Ok(build_candidate(
        source,
        meta.name,
        meta.description,
        warnings,
        skill_file,
        None,
    ))
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
