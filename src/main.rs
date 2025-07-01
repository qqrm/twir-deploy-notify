use regex::Regex;
use std::{env, fs, path::Path};

const TELEGRAM_LIMIT: usize = 4000;

/// Escape characters that have special meaning in Telegram MarkdownV2. The list
/// follows the official specification and includes all symbols that require a
/// leading backslash.
pub fn escape_markdown(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|'
            | '{' | '}' | '.' | '!' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Escape only the characters that may terminate a Telegram MarkdownV2 link.
/// This function is intended for URLs inside the `(url)` part of `[text](url)`
/// constructs. It leaves characters like `#` and `+` untouched so that such
/// links remain valid.
pub fn escape_markdown_url(url: &str) -> String {
    let mut escaped = String::with_capacity(url.len());
    for ch in url.chars() {
        match ch {
            '(' | ')' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub fn split_posts(text: &str, limit: usize) -> Vec<String> {
    let mut posts = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        // calculate length if we were to add this line
        let new_len = if current.is_empty() {
            line.len()
        } else {
            current.len() + 1 + line.len()
        };

        if new_len > limit && !current.is_empty() {
            posts.push(current.clone());
            current.clear();
        }

        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }

    if !current.is_empty() {
        posts.push(current);
    }

    posts
}

/// Convert the provided markdown input into numbered Telegram posts without
/// writing them to disk. The logic mirrors the one used by `main`.
pub fn generate_posts(mut input: String) -> Vec<String> {
    let mut output = String::new();

    let debug = env::var("DEBUG").is_ok();
    if debug {
        eprintln!("Debug mode enabled");
    }

    let title_re = Regex::new(r"(?m)^Title: (.+)$").unwrap();
    let number_re = Regex::new(r"(?m)^Number: (.+)$").unwrap();
    let date_re = Regex::new(r"(?m)^Date: (.+)$").unwrap();

    let title = title_re
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    let number = number_re
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    let date = date_re
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());

    if let Some(ref t) = title {
        output.push_str(&format!("**{}**", escape_markdown(t)));
    }
    if let Some(ref n) = number {
        output.push_str(&format!(" — \\#{}", escape_markdown(n)));
    }
    if let Some(ref d) = date {
        output.push_str(&format!(" — {}\n\n\\-\\-\\-\n\n", escape_markdown(d)));
    }

    let url = if let (Some(ref d), Some(ref n)) = (date.as_ref(), number.as_ref()) {
        let parts: Vec<&str> = d.split('-').collect();
        if parts.len() >= 3 {
            Some(format!(
                "https://this-week-in-rust.org/blog/{}/{}/{}/this-week-in-rust-{}/",
                parts[0], parts[1], parts[2], n
            ))
        } else {
            None
        }
    } else {
        None
    };

    if let Some(ref link) = url {
        if debug {
            eprintln!("detected URL: {link}");
        }
        input = input.replace(
            "_Полный выпуск: ссылка_",
            &format!(
                "Полный выпуск: [{}]({})",
                escape_markdown(link),
                escape_markdown_url(link)
            ),
        );
    }

    let section_re = Regex::new(r"^##+\s+(.+)$").unwrap();
    let header_link_re = Regex::new(r"^\[(.+?)\]\((.+?)\)$").unwrap();
    let link_re = Regex::new(r"^[*-] \[(.+?)\]\((.+?)\)").unwrap();
    let cotw_re = Regex::new(r"^\[(.+?)\]\((.+?)\)\s*[—-]\s*(.+)$").unwrap();

    let mut lines = input.lines().peekable();
    let mut current_section: Option<String> = None;
    let mut section_links: Vec<String> = Vec::new();
    let mut first_section = true;

    while let Some(line) = lines.next() {
        if debug {
            eprintln!("line: {line}");
        }
        if let Some(sec) = section_re.captures(line) {
            if debug {
                eprintln!("section header: {}", sec[1].trim());
            }
            if !section_links.is_empty() {
                if !first_section {
                    output.push('\n');
                } else {
                    first_section = false;
                }
                if let Some(title) = current_section.take() {
                    if let Some(caps) = header_link_re.captures(&title) {
                        output.push_str(&format!(
                            "**[{}]({})**\n",
                            escape_markdown(&caps[1]),
                            escape_markdown_url(&caps[2])
                        ));
                    } else {
                        output.push_str(&format!("**{}**\n", escape_markdown(&title)));
                    }
                    for link in &section_links {
                        output.push_str(link);
                        output.push('\n');
                    }
                }
                section_links.clear();
            }

            let title = sec[1].trim();
            if title == "Crate of the Week" {
                if debug {
                    eprintln!("processing Crate of the Week section");
                }
                if let Some(next_line) = lines.next() {
                    if let Some(caps) = cotw_re.captures(next_line) {
                        if !first_section {
                            output.push('\n');
                        } else {
                            first_section = false;
                        }
                        output.push_str("**Crate of the Week**\n");
                        output.push_str(&format!(
                            "\\- [{}]({}) — {}\n",
                            escape_markdown(&caps[1]),
                            escape_markdown_url(&caps[2]),
                            escape_markdown(&caps[3])
                        ));
                    }
                }
                current_section = None;
            } else {
                current_section = Some(title.to_string());
            }
            continue;
        }

        if let Some(caps) = link_re.captures(line) {
            if debug {
                eprintln!("link detected: [{}]({})", &caps[1], &caps[2]);
            }
            let title = &caps[1];
            let url = &caps[2];
            section_links.push(format!(
                "\\- [{}]({})",
                escape_markdown(title),
                escape_markdown_url(url)
            ));
        } else if debug {
            eprintln!("unrecognized line: {line}");
        }
    }

    if !section_links.is_empty() {
        if !first_section {
            output.push('\n');
        }
        if let Some(title) = current_section.take() {
            if let Some(caps) = header_link_re.captures(&title) {
                output.push_str(&format!(
                    "**[{}]({})**\n",
                    escape_markdown(&caps[1]),
                    escape_markdown_url(&caps[2])
                ));
            } else {
                output.push_str(&format!("**{}**\n", escape_markdown(&title)));
            }
            for link in &section_links {
                output.push_str(link);
                output.push('\n');
            }
        }
    }

    output.push_str("\n\\-\\-\\-\n\n");
    if let Some(link) = url {
        output.push_str(&format!(
            "Полный выпуск: [{}]({})\n",
            escape_markdown(&link),
            escape_markdown_url(&link)
        ));
    }

    if debug {
        eprintln!("final output before split:\n{output}");
    }
    let raw_posts = split_posts(&output, TELEGRAM_LIMIT);
    let total = raw_posts.len();
    raw_posts
        .into_iter()
        .enumerate()
        .map(|(i, post)| format!("*Часть {}/{}*\n{}", i + 1, total, post))
        .collect()
}

/// Write provided posts to files named `output_N.md` under `dir`.
pub fn write_posts(posts: &[String], dir: &Path) -> std::io::Result<()> {
    for (i, post) in posts.iter().enumerate() {
        let file_name = dir.join(format!("output_{}.md", i + 1));
        fs::write(&file_name, post)?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).map(String::as_str).unwrap_or("input.md");
    let input = fs::read_to_string(input_path)?;
    let posts = generate_posts(input);
    write_posts(&posts, Path::new("."))?;
    for (i, _) in posts.iter().enumerate() {
        println!("Generated output_{}.md", i + 1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn split_posts_basic() {
        let text = "aaa\nbbb\nccc";
        let parts = split_posts(text, 7);
        assert_eq!(parts, vec!["aaa\nbbb", "ccc"]);
    }

    #[test]
    fn generate_and_write_files() {
        let input =
            "Title: Test\nNumber: 1\nDate: 2024-01-01\n\n## News\n- [Link](https://example.com)\n";
        let posts = generate_posts(input.to_string());
        let mut dir = std::env::temp_dir();
        dir.push("twir_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        write_posts(&posts, &dir).unwrap();
        let first = dir.join("output_1.md");
        assert!(first.exists());
        let content = fs::read_to_string(first).unwrap();
        assert!(content.contains("*Часть 1/1*"));
        assert!(content.contains("**News**"));
        let _ = fs::remove_dir_all(&dir);
    }
}
