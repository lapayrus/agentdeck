use serde_yaml::Value;

pub struct Frontmatter {
    pub data: Value,
    pub body: String,
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

pub struct AgentMeta {
    pub name: String,
    pub description: Option<String>,
    pub warnings: Vec<String>,
    pub skip_reason: Option<String>,
}

pub fn parse_agent_markdown(text: &str, fallback_name: &str) -> Result<AgentMeta, String> {
    let Some(frontmatter) = split_frontmatter(text)? else {
        return Ok(AgentMeta {
            name: fallback_name.to_string(),
            description: None,
            warnings: vec!["No YAML frontmatter; this ecosystem will not load the agent.".into()],
            skip_reason: Some("missing frontmatter".into()),
        });
    };

    let mut warnings = Vec::new();
    let name = match string_field(&frontmatter.data, "name") {
        Some(name) => name,
        None => {
            return Ok(AgentMeta {
                name: fallback_name.to_string(),
                description: None,
                warnings: vec!["The agent has no name and will be skipped.".into()],
                skip_reason: Some("missing name".into()),
            })
        }
    };

    if name.len() > 256 {
        return Ok(AgentMeta {
            name,
            description: None,
            warnings: vec!["The agent name is longer than 256 characters.".into()],
            skip_reason: Some("invalid name".into()),
        });
    }
    if name.contains(':') || name.starts_with('-') {
        return Ok(AgentMeta {
            name,
            description: None,
            warnings: vec!["The agent name contains a colon or starts with a hyphen.".into()],
            skip_reason: Some("invalid name".into()),
        });
    }
    if name != fallback_name {
        warnings.push(format!(
            "The name \"{name}\" differs from the file name \"{fallback_name}\"."
        ));
    }

    let description = string_field(&frontmatter.data, "description");
    if description.is_none() {
        warnings.push(
            "The agent has no description; Claude Code skips agents without one.".into(),
        );
    }

    Ok(AgentMeta {
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
}
