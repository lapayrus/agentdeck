pub fn strip_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut characters = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(character) = characters.next() {
        if in_string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => {
                in_string = true;
                output.push(character);
            }
            '/' => match characters.peek() {
                Some('/') => {
                    characters.next();
                    while let Some(&next) = characters.peek() {
                        if next == '\n' {
                            break;
                        }
                        characters.next();
                    }
                }
                Some('*') => {
                    characters.next();
                    let mut previous = '\0';
                    while let Some(next) = characters.next() {
                        if previous == '*' && next == '/' {
                            break;
                        }
                        if next == '\n' {
                            output.push('\n');
                        }
                        previous = next;
                    }
                }
                _ => output.push('/'),
            },
            _ => output.push(character),
        }
    }
    output
}

fn without_trailing_commas(input: &str) -> String {
    let characters: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];
        if in_string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if character == '"' {
            in_string = true;
            output.push(character);
            index += 1;
            continue;
        }
        if character == ',' {
            let mut lookahead = index + 1;
            while lookahead < characters.len() && characters[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < characters.len()
                && (characters[lookahead] == '}' || characters[lookahead] == ']')
            {
                index += 1;
                continue;
            }
        }
        output.push(character);
        index += 1;
    }
    output
}

pub fn parse(input: &str) -> Result<serde_json::Value, String> {
    let cleaned = without_trailing_commas(&strip_comments(input));
    serde_json::from_str(&cleaned).map_err(|error| format!("invalid JSON: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_line_and_block_comments() {
        let text = r#"{
  // a line comment
  "a": 1, /* block */
  "b": 2
}"#;
        let value = parse(text).unwrap();
        assert_eq!(value["a"], 1);
        assert_eq!(value["b"], 2);
    }

    #[test]
    fn keeps_comment_markers_inside_strings() {
        let text = r#"{"url": "https://example.com/a//b", "note": "/* keep */"}"#;
        let value = parse(text).unwrap();
        assert_eq!(value["url"], "https://example.com/a//b");
        assert_eq!(value["note"], "/* keep */");
    }

    #[test]
    fn tolerates_trailing_commas() {
        let text = r#"{ "a": [1, 2,], "b": 3, }"#;
        let value = parse(text).unwrap();
        assert_eq!(value["a"][1], 2);
    }
}
