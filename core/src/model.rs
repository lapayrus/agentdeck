use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Ecosystem {
    #[serde(rename = "claude")]
    Claude,
    #[serde(rename = "codex")]
    Codex,
    #[serde(rename = "opencode")]
    Opencode,
    #[serde(rename = "commandcode")]
    CommandCode,
}

impl Ecosystem {
    pub const ALL: [Ecosystem; 4] = [
        Ecosystem::Claude,
        Ecosystem::Codex,
        Ecosystem::Opencode,
        Ecosystem::CommandCode,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Ecosystem::Claude => "Claude Code",
            Ecosystem::Codex => "Codex",
            Ecosystem::Opencode => "OpenCode",
            Ecosystem::CommandCode => "Command Code",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactType {
    Agent,
    Skill,
    Mcp,
    Hook,
    Command,
    Plugin,
    Memory,
    Settings,
}

impl ArtifactType {
    pub fn is_managed(&self) -> bool {
        matches!(
            self,
            ArtifactType::Agent | ArtifactType::Skill | ArtifactType::Mcp
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    Global,
    Project,
    ProjectLocal,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ReloadHint {
    Live,
    NextTurn,
    NextSession,
    Restart,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShadowedArtifact {
    pub name: String,
    pub path: String,
    pub scope: Scope,
    pub ecosystem: Ecosystem,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub ecosystem: Ecosystem,
    pub artifact_type: ArtifactType,
    pub scope: Scope,
    pub primary_path: String,
    pub source_paths: Vec<String>,
    pub enabled: Option<bool>,
    pub ecosystems: Vec<Ecosystem>,
    pub shadowed: Vec<ShadowedArtifact>,
    pub warnings: Vec<String>,
    pub managed: bool,
    pub can_disable: bool,
    pub can_remove: bool,
    pub disable_mechanism: Option<String>,
    pub transport: Option<String>,
    pub command: Option<String>,
    pub url: Option<String>,
    pub key_path: Option<String>,
    pub reload: ReloadHint,
}

impl Artifact {
    pub fn new(
        ecosystem: Ecosystem,
        artifact_type: ArtifactType,
        scope: Scope,
        name: impl Into<String>,
        primary_path: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let primary_path = primary_path.into();
        let id = format!(
            "{}:{}:{:?}:{}",
            ecosystem_key(ecosystem),
            type_key(artifact_type),
            scope,
            primary_path.to_lowercase()
        );
        Self {
            id,
            name,
            description: None,
            ecosystem,
            artifact_type,
            scope,
            primary_path,
            source_paths: Vec::new(),
            enabled: Some(true),
            ecosystems: vec![ecosystem],
            shadowed: Vec::new(),
            warnings: Vec::new(),
            managed: artifact_type.is_managed(),
            can_disable: false,
            can_remove: artifact_type.is_managed(),
            disable_mechanism: None,
            transport: None,
            command: None,
            url: None,
            key_path: None,
            reload: ReloadHint::Unknown,
        }
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

fn type_key(artifact_type: ArtifactType) -> &'static str {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EcosystemDetection {
    pub ecosystem: Ecosystem,
    pub detected: bool,
    pub version: Option<String>,
    pub checked_paths: Vec<String>,
    pub capabilities: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub artifacts: Vec<Artifact>,
    pub detections: Vec<EcosystemDetection>,
    pub warnings: Vec<String>,
    pub project_root: Option<String>,
    pub is_git_repository: bool,
    pub scanned_at: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub project_root: Option<String>,
}
