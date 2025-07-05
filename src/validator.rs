use std::collections::VecDeque;

/// Validate that the provided text conforms to Telegram Markdown v2 rules
/// used in this project. Returns `Ok(())` if valid, otherwise an error
/// message describing the first encountered problem.
pub fn validate_telegram_markdown(text: &str) -> Result<(), String> {
    let chars: Vec<char> = text.chars().collect();
    let mut stack: VecDeque<&str> = VecDeque::new();
    if let Some(&first) = chars.first() {
        if matches!(first, '-' | '>' | '#' | '+' | '=' | '{' | '}' | '.' | '!') {
            return Err("Post starts with reserved character".to_string());
        }
    }
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '*' => {
                let token = if i + 1 < chars.len() && chars[i + 1] == '*' {
                    i += 1;
                    "**"
                } else {
                    "*"
                };
                toggle_token(token, &mut stack)?;
            }
            '_' => {
                let token = if i + 1 < chars.len() && chars[i + 1] == '_' {
                    i += 1;
                    "__"
                } else {
                    "_"
                };
                toggle_token(token, &mut stack)?;
            }
            '~' => toggle_token("~", &mut stack)?,
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    i += 1;
                    toggle_token("||", &mut stack)?;
                }
            }
            '`' => {
                let token = if i + 2 < chars.len() && chars[i + 1] == '`' && chars[i + 2] == '`' {
                    i += 2;
                    "```"
                } else {
                    "`"
                };
                toggle_token(token, &mut stack)?;
            }
            '[' => {
                let mut j = i + 1;
                while j < chars.len() {
                    if chars[j] == '\\' {
                        j += 1;
                    } else if chars[j] == ']' {
                        break;
                    }
                    j += 1;
                }
                if j >= chars.len() || j + 1 >= chars.len() || chars[j + 1] != '(' {
                    return Err(format!("Unmatched [ at {i}"));
                }
                j += 2;
                while j < chars.len() {
                    if chars[j] == '\\' {
                        j += 1;
                    } else if chars[j] == ')' {
                        break;
                    }
                    j += 1;
                }
                if j >= chars.len() {
                    return Err(format!("Unmatched [ at {i}"));
                }
                i = j;
            }
            '\\' => {
                i += 1; // skip escaped char
            }
            '-' | '>' | '#' | '+' | '=' | '{' | '}' | '.' | '!' => {
                let prev = if i == 0 { None } else { Some(chars[i - 1]) };
                if prev != Some('\\') {
                    return Err(format!("Unescaped {ch} at {i}"));
                }
            }
            _ => {}
        }
        i += 1;
    }
    if let Some(tok) = stack.pop_back() {
        return Err(format!("Unclosed {tok} entity"));
    }
    Ok(())
}

fn toggle_token<'a>(token: &'a str, stack: &mut VecDeque<&'a str>) -> Result<(), String> {
    if let Some(last) = stack.back() {
        if *last == token {
            stack.pop_back();
            return Ok(());
        }
        if stack.contains(&token) {
            return Err(format!("Mismatched {token} entity"));
        }
    }
    stack.push_back(token);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_validation() {
        validate_telegram_markdown("*bold*").unwrap();
        assert!(validate_telegram_markdown("*bold").is_err());
    }

    #[test]
    fn rejects_unescaped_dash() {
        assert!(validate_telegram_markdown("some - text").is_err());
    }

    #[test]
    fn accepts_escaped_dash() {
        assert!(validate_telegram_markdown("some \\- text").is_ok());
    }
}
