use serde_yaml::Value;

pub struct Frontmatter {
    pub data: Value,
    pub body: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DescriptionRule {
    Optional,
    Warn,
    Required,
}

pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub warnings: Vec<String>,
    pub skip_reason: Option<String>,
}

pub fn split_frontmatter(text: &str) -> Result<Option<Frontmatter>, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text.split_inclusive('\n');
    let first = match lines.next() {
        Some(line) => line.trim_end_matches(['\r', '\n']),
        None => return Ok(None),
    };
    if first.trim() != "---" {
        return Ok(None);
    }
    let mut offset = first.len();
    if text[offset..].starts_with('\n') {
        offset += 1;
    }
    let rest = &text[offset..];
    let mut consumed = 0usize;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.trim() == "---" {
            let yaml = &rest[..consumed];
            let body = &rest[consumed + line.len()..];
            let data = serde_yaml::from_str::<Value>(yaml)
                .map_err(|error| format!("invalid YAML frontmatter: {error}"))?;
            return Ok(Some(Frontmatter {
                data,
                body: body.to_string(),
            }));
        }
        consumed += line.len();
    }
    Err("frontmatter starts with --- but never closes".to_string())
}

pub fn string_field(data: &Value, key: &str) -> Option<String> {
    data.get(key).and_then(|value| match value {
        Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_string()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    })
}

fn skipped(name: String, warning: String, reason: &str) -> Metadata {
    Metadata {
        name,
        description: None,
        warnings: vec![warning],
        skip_reason: Some(reason.to_string()),
    }
}

pub fn parse_markdown(
    text: &str,
    fallback_name: &str,
    require_name: bool,
    description_rule: DescriptionRule,
    label: &str,
) -> Result<Metadata, String> {
    let Some(frontmatter) = split_frontmatter(text)? else {
        return Ok(skipped(
            fallback_name.to_string(),
            format!("No YAML frontmatter; {label} will not load this entry."),
            "missing frontmatter",
        ));
    };

    let mut warnings = Vec::new();
    let declared = string_field(&frontmatter.data, "name");
    let name = match &declared {
        Some(name) => name.clone(),
        None if require_name => {
            return Ok(skipped(
                fallback_name.to_string(),
                format!("No name is declared; {label} will skip this entry."),
                "missing name",
            ))
        }
        None => fallback_name.to_string(),
    };

    if name.len() > 256 {
        return Ok(skipped(
            name,
            "The name is longer than 256 characters.".to_string(),
            "invalid name",
        ));
    }
    if name.contains(':') || name.starts_with('-') {
        return Ok(skipped(
            name,
            "The name contains a colon or starts with a hyphen.".to_string(),
            "invalid name",
        ));
    }
    if declared.is_some() && name != fallback_name {
        warnings.push(format!(
            "The declared name \"{name}\" differs from the file name \"{fallback_name}\"."
        ));
    }

    let description = string_field(&frontmatter.data, "description");
    match (description.is_some(), description_rule) {
        (false, DescriptionRule::Required) => {
            return Ok(skipped(
                name,
                format!("No description is declared; {label} will not advertise this entry."),
                "missing description",
            ))
        }
        (false, DescriptionRule::Warn) => warnings
            .push(format!("No description is declared; {label} may skip this entry.")),
        _ => {}
    }

    Ok(Metadata {
        name,
        description,
        warnings,
        skip_reason: None,
    })
}

pub fn parse_agent_markdown(text: &str, fallback_name: &str) -> Result<Metadata, String> {
    parse_markdown(
        text,
        fallback_name,
        true,
        DescriptionRule::Warn,
        "Claude Code",
    )
}

pub fn parse_agent_toml(text: &str, path_label: &str) -> Result<Metadata, String> {
    let document = text
        .parse::<toml_edit::DocumentMut>()
        .map_err(|error| format!("invalid TOML: {error}"))?;
    let name = document
        .get("name")
        .and_then(|item| item.as_str())
        .map(|value| value.to_string());
    let description = document
        .get("description")
        .and_then(|item| item.as_str())
        .map(|value| value.to_string());
    let instructions = document
        .get("developer_instructions")
        .and_then(|item| item.as_str())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);

    let Some(name) = name else {
        return Ok(skipped(
            path_label.to_string(),
            "The agent TOML has no name and will be skipped.".to_string(),
            "missing name",
        ));
    };
    if description.is_none() {
        return Ok(skipped(
            name,
            "The agent TOML has no description and will be skipped.".to_string(),
            "missing description",
        ));
    }
    let mut warnings = Vec::new();
    if !instructions {
        warnings.push("The agent TOML has no developer_instructions.".into());
    }

    Ok(Metadata {
        name,
        description,
        warnings,
        skip_reason: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_frontmatter_and_body() {
        let text = "---\nname: reviewer\ndescription: Reviews code\n---\n\nBody here.\n";
        let parsed = split_frontmatter(text).unwrap().unwrap();
        assert_eq!(string_field(&parsed.data, "name").unwrap(), "reviewer");
        assert!(parsed.body.contains("Body here."));
    }

    #[test]
    fn rejects_unterminated_frontmatter() {
        assert!(split_frontmatter("---\nname: x\n").is_err());
    }

    #[test]
    fn parses_required_agent_fields() {
        let text = "---\nname: reviewer\ndescription: Reviews code\n---\nBody";
        let meta = parse_agent_markdown(text, "reviewer").unwrap();
        assert_eq!(meta.name, "reviewer");
        assert_eq!(meta.description.as_deref(), Some("Reviews code"));
        assert!(meta.skip_reason.is_none());
        assert!(meta.warnings.is_empty());
    }

    #[test]
    fn warns_when_description_is_missing() {
        let text = "---\nname: reviewer\n---\nBody";
        let meta = parse_agent_markdown(text, "reviewer").unwrap();
        assert!(meta.skip_reason.is_none());
        assert_eq!(meta.warnings.len(), 1);
    }

    #[test]
    fn skips_an_agent_without_a_name() {
        let text = "---\ndescription: Reviews code\n---\nBody";
        let meta = parse_agent_markdown(text, "reviewer").unwrap();
        assert_eq!(meta.skip_reason.as_deref(), Some("missing name"));
    }

    #[test]
    fn requires_a_description_when_the_ecosystem_does() {
        let text = "---\nname: reviewer\n---\nBody";
        let meta = parse_markdown(
            text,
            "reviewer",
            true,
            DescriptionRule::Required,
            "OpenCode",
        )
        .unwrap();
        assert_eq!(meta.skip_reason.as_deref(), Some("missing description"));
    }

    #[test]
    fn reads_codex_toml_agents() {
        let text = "name = \"reviewer\"\ndescription = \"Reviews code\"\ndeveloper_instructions = \"Be strict\"\n";
        let meta = parse_agent_toml(text, "reviewer").unwrap();
        assert_eq!(meta.name, "reviewer");
        assert!(meta.skip_reason.is_none());
    }

    #[test]
    fn codex_toml_without_a_name_is_skipped() {
        let text = "description = \"Reviews code\"\n";
        let meta = parse_agent_toml(text, "reviewer").unwrap();
        assert_eq!(meta.skip_reason.as_deref(), Some("missing name"));
    }
}
