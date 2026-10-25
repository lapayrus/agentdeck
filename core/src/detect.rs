use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::model::Ecosystem;
use crate::roots::Roots;

fn cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn parse_version(output: &str) -> Option<String> {
    for token in output.split(|character: char| character.is_whitespace() || character == ',') {
        let cleaned = token.trim_start_matches('v').trim_matches(|character: char| {
            !character.is_ascii_alphanumeric() && character != '.' && character != '-'
        });
        let core = cleaned
            .chars()
            .take_while(|character| character.is_ascii_digit() || *character == '.')
            .collect::<String>();
        if core.matches('.').count() >= 1 && core.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            return Some(core);
        }
    }
    None
}

pub fn capabilities(ecosystem: Ecosystem, version: Option<&str>) -> Vec<String> {
    let Some(version) = version else {
        return Vec::new();
    };
    let major: u64 = version
        .split('.')
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let minor: u64 = version
        .split('.')
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let mut capabilities: Vec<String> = Vec::new();
    match ecosystem {
        Ecosystem::Claude => {
            if (major, minor) >= (2, 1) {
                capabilities.extend(
                    ["disable-agent", "disable-skill", "disable-mcp", "disable-quarantine"]
                        .iter()
                        .map(|value| value.to_string()),
                );
            }
        }
        Ecosystem::Codex => {
            if major >= 1 {
                capabilities.push("disable-skill".into());
                capabilities.push("disable-mcp".into());
                capabilities.push("disable-quarantine".into());
            }
        }
        Ecosystem::Opencode => {
            if major >= 2 {
                capabilities.extend(
                    ["disable-agent", "disable-skill", "disable-mcp"]
                        .iter()
                        .map(|value| value.to_string()),
                );
            }
            capabilities.push("disable-quarantine".into());
        }
        Ecosystem::CommandCode => {
            if major >= 1 {
                capabilities.push("disable-skill".into());
                capabilities.push("disable-mcp".into());
            }
            capabilities.push("disable-quarantine".into());
        }
    }
    capabilities
}

fn candidates(ecosystem: Ecosystem, roots: &Roots) -> Vec<PathBuf> {
    if let Some(path) = roots.binaries.get(&ecosystem) {
        return vec![path.clone()];
    }
    let mut paths = Vec::new();
    match ecosystem {
        Ecosystem::Claude => {
            paths.push(roots.home.join(".local").join("bin").join("claude.exe"));
            paths.push(roots.home.join(".local").join("bin").join("claude"));
        }
        Ecosystem::Codex => {
            let bin = roots
                .home
                .join("AppData")
                .join("Local")
                .join("OpenAI")
                .join("Codex")
                .join("bin");
            if let Ok(entries) = std::fs::read_dir(bin) {
                for entry in entries.flatten() {
                    let candidate = entry.path().join("codex.exe");
                    if candidate.is_file() {
                        paths.push(candidate);
                    }
                }
            }
            paths.push(PathBuf::from("codex.exe"));
        }
        Ecosystem::Opencode => {
            paths.push(
                roots
                    .home
                    .join("AppData")
                    .join("Local")
                    .join("Programs")
                    .join("@opencodedesktop")
                    .join("resources")
                    .join("opencode-cli.exe"),
            );
            paths.push(PathBuf::from("opencode.exe"));
        }
        Ecosystem::CommandCode => {
            paths.push(PathBuf::from("cmdc.cmd"));
            paths.push(PathBuf::from("cmdc.exe"));
        }
    }
    paths
}

fn run_version(binary: &PathBuf) -> Option<String> {
    let key = binary.to_string_lossy().to_lowercase();
    if let Ok(cache) = cache().lock() {
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }
    let mut command = Command::new(binary);
    command.arg("--version");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let output = command.output().ok();
    let version = output.and_then(|output| {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        parse_version(&stdout).or_else(|| parse_version(&stderr))
    });
    if let Ok(mut cache) = cache().lock() {
        cache.insert(key, version.clone());
    }
    version
}

pub fn probe(ecosystem: Ecosystem, roots: &Roots) -> Option<String> {
    if let Some(version) = roots.version_overrides.get(&ecosystem) {
        return Some(version.clone());
    }
    for binary in candidates(ecosystem, roots) {
        if binary.is_absolute() && !binary.is_file() {
            continue;
        }
        if let Some(version) = run_version(&binary) {
            return Some(version);
        }
    }
    None
}

pub fn supports(ecosystem: Ecosystem, version: Option<&str>, capability: &str) -> bool {
    capabilities(ecosystem, version)
        .iter()
        .any(|entry| entry == capability)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_version_banners() {
        assert_eq!(parse_version("claude 2.1.215"), Some("2.1.215".into()));
        assert_eq!(
            parse_version("codex-cli 0.164.0-alpha.11.1"),
            Some("0.164.0".into())
        );
        assert_eq!(parse_version("2.1.4"), Some("2.1.4".into()));
        assert_eq!(parse_version("no digits"), None);
    }

    #[test]
    fn gates_capabilities_by_version() {
        assert!(supports(Ecosystem::Claude, Some("2.1.215"), "disable-agent"));
        assert!(!supports(Ecosystem::Claude, Some("1.9.0"), "disable-agent"));
        assert!(supports(Ecosystem::Codex, Some("1.0.0"), "disable-mcp"));
        assert!(!supports(Ecosystem::Opencode, Some("1.1.65"), "disable-agent"));
        assert!(supports(Ecosystem::Opencode, Some("2.1.4"), "disable-agent"));
        assert!(capabilities(Ecosystem::Claude, None).is_empty());
    }
}
