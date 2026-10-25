#[derive(Clone, Debug)]
pub struct Node {
    pub start: usize,
    pub end: usize,
    pub kind: Kind,
}

#[derive(Clone, Debug)]
pub enum Kind {
    Object(Vec<Member>),
    Array(Vec<Node>),
    Scalar,
}

#[derive(Clone, Debug)]
pub struct Member {
    pub key: String,
    pub key_start: usize,
    pub value: Node,
}

impl Node {
    pub fn members(&self) -> Option<&Vec<Member>> {
        match &self.kind {
            Kind::Object(members) => Some(members),
            _ => None,
        }
    }

    pub fn member(&self, key: &str) -> Option<&Member> {
        self.members()
            .and_then(|members| members.iter().find(|member| member.key == key))
    }

    pub fn items(&self) -> Option<&Vec<Node>> {
        match &self.kind {
            Kind::Array(items) => Some(items),
            _ => None,
        }
    }
}

pub fn parse(text: &str) -> Result<Node, String> {
    let mut parser = Parser {
        data: text.as_bytes(),
        pos: 0,
    };
    let node = parser.parse_value()?;
    parser.skip();
    if parser.pos != parser.data.len() {
        return Err(format!(
            "unexpected trailing content at byte {}",
            parser.pos
        ));
    }
    Ok(node)
}

pub fn find<'a>(root: &'a Node, path: &[&str]) -> Option<&'a Node> {
    let mut current = root;
    for key in path {
        current = &current.member(key)?.value;
    }
    Some(current)
}

struct Parser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    fn skip(&mut self) {
        loop {
            while self
                .peek()
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                self.pos += 1;
            }
            if self.peek() == Some(b'/') && self.data.get(self.pos + 1) == Some(&b'/') {
                self.pos += 2;
                while self.peek().is_some_and(|byte| byte != b'\n') {
                    self.pos += 1;
                }
                continue;
            }
            if self.peek() == Some(b'/') && self.data.get(self.pos + 1) == Some(&b'*') {
                self.pos += 2;
                while self.pos + 1 < self.data.len()
                    && !(self.data[self.pos] == b'*' && self.data[self.pos + 1] == b'/')
                {
                    self.pos += 1;
                }
                self.pos = (self.pos + 2).min(self.data.len());
                continue;
            }
            break;
        }
    }

    fn parse_value(&mut self) -> Result<Node, String> {
        self.skip();
        let start = self.pos;
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => {
                self.parse_string()?;
                Ok(Node {
                    start,
                    end: self.pos,
                    kind: Kind::Scalar,
                })
            }
            Some(_) => {
                while let Some(byte) = self.peek() {
                    if byte == b','
                        || byte == b'}'
                        || byte == b']'
                        || byte.is_ascii_whitespace()
                    {
                        break;
                    }
                    self.pos += 1;
                }
                Ok(Node {
                    start,
                    end: self.pos,
                    kind: Kind::Scalar,
                })
            }
            None => Err("unexpected end of document".into()),
        }
    }

    fn parse_object(&mut self) -> Result<Node, String> {
        let start = self.pos;
        self.pos += 1;
        let mut members = Vec::new();
        loop {
            self.skip();
            match self.peek() {
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'"') => {
                    let key_start = self.pos;
                    let key = self.parse_string()?;
                    self.skip();
                    if self.peek() != Some(b':') {
                        return Err(format!("expected ':' after key \"{key}\""));
                    }
                    self.pos += 1;
                    let value = self.parse_value()?;
                    members.push(Member {
                        key,
                        key_start,
                        value,
                    });
                }
                Some(byte) => {
                    return Err(format!(
                        "unexpected character '{}' in object",
                        byte as char
                    ))
                }
                None => return Err("unterminated object".into()),
            }
        }
        Ok(Node {
            start,
            end: self.pos,
            kind: Kind::Object(members),
        })
    }

    fn parse_array(&mut self) -> Result<Node, String> {
        let start = self.pos;
        self.pos += 1;
        let mut items = Vec::new();
        loop {
            self.skip();
            match self.peek() {
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                Some(b',') => {
                    self.pos += 1;
                }
                Some(_) => items.push(self.parse_value()?),
                None => return Err("unterminated array".into()),
            }
        }
        Ok(Node {
            start,
            end: self.pos,
            kind: Kind::Array(items),
        })
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.peek() != Some(b'"') {
            return Err("expected a string".into());
        }
        self.pos += 1;
        let mut output = String::new();
        while let Some(byte) = self.peek() {
            if byte == b'"' {
                self.pos += 1;
                return Ok(output);
            }
            if byte == b'\\' {
                self.pos += 1;
                if let Some(escaped) = self.peek() {
                    output.push(match escaped {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        other => other as char,
                    });
                    self.pos += 1;
                }
                continue;
            }
            if byte.is_ascii() {
                output.push(byte as char);
                self.pos += 1;
            } else {
                let rest = std::str::from_utf8(&self.data[self.pos..])
                    .map_err(|_| "invalid UTF-8 in string".to_string())?;
                let character = rest.chars().next().ok_or("invalid string")?;
                output.push(character);
                self.pos += character.len_utf8();
            }
        }
        Err("unterminated string".into())
    }
}

fn line_indent(text: &str, position: usize) -> String {
    let line_start = text[..position].rfind('\n').map(|index| index + 1).unwrap_or(0);
    text[line_start..position]
        .chars()
        .take_while(|character| *character == ' ' || *character == '\t')
        .collect()
}

fn last_significant(text: &str, before: usize) -> Option<u8> {
    text.as_bytes()[..before]
        .iter()
        .rev()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

pub fn set_member(
    text: &str,
    parent_path: &[&str],
    key: &str,
    value_json: &str,
) -> Result<String, String> {
    let root = parse(text)?;
    let parent = find(&root, parent_path)
        .ok_or_else(|| format!("{} was not found in the document", parent_path.join(".")))?;
    let members = parent
        .members()
        .ok_or_else(|| format!("{} is not an object", parent_path.join(".")))?;
    let key_json = serde_json::to_string(key).map_err(|error| error.to_string())?;

    if let Some(member) = members.iter().find(|member| member.key == key) {
        let mut output = String::with_capacity(text.len() + value_json.len());
        output.push_str(&text[..member.value.start]);
        output.push_str(value_json);
        output.push_str(&text[member.value.end..]);
        return Ok(output);
    }

    let close = parent.end - 1;
    let close_indent = line_indent(text, close);
    let child_indent = format!("{close_indent}  ");
    let needs_comma = !members.is_empty()
        && last_significant(text, close) != Some(b',');
    let prefix = if needs_comma { "," } else { "" };
    let separator = if members.is_empty() { "" } else { "\n" };
    let insertion = format!(
        "{prefix}{separator}{child_indent}{key_json}: {value_json}\n{close_indent}"
    );
    let mut output = String::with_capacity(text.len() + insertion.len());
    output.push_str(&text[..close]);
    output.push_str(&insertion);
    output.push_str(&text[close..]);
    Ok(output)
}

pub fn remove_member(
    text: &str,
    parent_path: &[&str],
    key: &str,
) -> Result<String, String> {
    let root = parse(text)?;
    let parent = find(&root, parent_path)
        .ok_or_else(|| format!("{} was not found in the document", parent_path.join(".")))?;
    let member = parent
        .member(key)
        .ok_or_else(|| format!("{key} was not found"))?;
    Ok(remove_span(text, member.key_start, member.value.end))
}

fn remove_span(text: &str, start: usize, end: usize) -> String {
    let mut start = start;
    let mut end = end;
    while start > 0 && text.as_bytes()[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    if start > 0 && text.as_bytes()[start - 1] == b',' {
        start -= 1;
    } else {
        let mut cursor = end;
        while cursor < text.len() && text.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor < text.len() && text.as_bytes()[cursor] == b',' {
            end = cursor + 1;
        }
    }
    let mut output = String::with_capacity(text.len());
    output.push_str(&text[..start]);
    output.push_str(&text[end..]);
    output
}

pub fn ensure_object(text: &str, path: &[&str]) -> Result<String, String> {
    if path.is_empty() {
        return Ok(text.to_string());
    }
    let root = parse(text)?;
    if find(&root, path).is_some() {
        return Ok(text.to_string());
    }
    let (key, parent_path) = path.split_last().unwrap();
    set_member(text, parent_path, key, "{}")
}

pub fn ensure_array_contains(
    text: &str,
    path: &[&str],
    value_json: &str,
) -> Result<String, String> {
    let root = parse(text)?;
    if let Some(node) = find(&root, path) {
        let items = node
            .items()
            .ok_or_else(|| format!("{} is not an array", path.join(".")))?;
        if items.iter().any(|item| {
            text[item.start..item.end].trim() == value_json.trim()
        }) {
            return Ok(text.to_string());
        }
        let close = node.end - 1;
        let close_indent = line_indent(text, close);
        let child_indent = format!("{close_indent}  ");
        let needs_comma = !items.is_empty() && last_significant(text, close) != Some(b',');
        let prefix = if needs_comma { "," } else { "" };
        let separator = if items.is_empty() { "" } else { "\n" };
        let insertion =
            format!("{prefix}{separator}{child_indent}{value_json}\n{close_indent}");
        let mut output = String::with_capacity(text.len() + insertion.len());
        output.push_str(&text[..close]);
        output.push_str(&insertion);
        output.push_str(&text[close..]);
        return Ok(output);
    }
    let (key, parent_path) = path.split_last().unwrap();
    set_member(text, parent_path, key, &format!("[{value_json}]"))
}

pub fn remove_array_value(
    text: &str,
    path: &[&str],
    value_json: &str,
) -> Result<String, String> {
    let root = parse(text)?;
    let Some(node) = find(&root, path) else {
        return Ok(text.to_string());
    };
    let items = node
        .items()
        .ok_or_else(|| format!("{} is not an array", path.join(".")))?;
    let Some(item) = items
        .iter()
        .find(|item| text[item.start..item.end].trim() == value_json.trim())
    else {
        return Ok(text.to_string());
    };
    Ok(remove_span(text, item.start, item.end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_a_value_without_touching_comments() {
        let text = "{\n  // keep me\n  \"a\": 1\n}\n";
        let output = set_member(text, &[], "a", "2").unwrap();
        assert!(output.contains("// keep me"));
        assert!(output.contains("\"a\": 2"));
    }

    #[test]
    fn adds_a_member_to_an_object() {
        let text = "{\n  \"a\": 1\n}\n";
        let output = set_member(text, &[], "b", "true").unwrap();
        let parsed = parse(&output).unwrap();
        assert_eq!(parsed.member("b").unwrap().value.end > 0, true);
        assert_eq!(parsed.members().unwrap().len(), 2);
    }

    #[test]
    fn adds_a_member_to_an_empty_object() {
        let output = set_member("{}", &[], "a", "1").unwrap();
        assert_eq!(parse(&output).unwrap().members().unwrap().len(), 1);
    }

    #[test]
    fn appends_to_an_array_and_creates_it() {
        let text = "{\n  \"deny\": [\"Agent(a)\"]\n}";
        let output = ensure_array_contains(text, &["deny"], "\"Agent(b)\"").unwrap();
        let root = parse(&output).unwrap();
        assert_eq!(root.member("deny").unwrap().value.items().unwrap().len(), 2);

        let created = ensure_array_contains("{}", &["deny"], "\"Agent(b)\"").unwrap();
        let root = parse(&created).unwrap();
        assert_eq!(root.member("deny").unwrap().value.items().unwrap().len(), 1);
    }

    #[test]
    fn removes_a_member_and_keeps_valid_json() {
        let text = "{\n  \"a\": 1,\n  \"b\": 2\n}";
        let output = remove_member(text, &[], "a").unwrap();
        let root = parse(&output).unwrap();
        assert_eq!(root.members().unwrap().len(), 1);
        assert!(root.member("a").is_none());
    }

    #[test]
    fn navigates_nested_objects() {
        let text = "{\"projects\": {\"C:/repo\": {\"mcpServers\": {\"x\": {}}}}}";
        let root = parse(text).unwrap();
        assert!(find(&root, &["projects", "C:/repo", "mcpServers", "x"]).is_some());
    }
}
