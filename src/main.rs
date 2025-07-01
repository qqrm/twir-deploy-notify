use regex::Regex;
use std::{env, fs, path::Path};

const TELEGRAM_LIMIT: usize = 4000;

/// Escape characters for MarkdownV2
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

/// Escape characters for the URL part in [text](url)
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

/// Convert Markdown-formatted text into plain text with URLs in parentheses
pub fn markdown_to_plain(text: &str) -> String {
    let without_escapes = text.replace('\\', "");
    let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    let replaced = link_re.replace_all(&without_escapes, "$1 ($2)");
    replaced.replace('*', "")
}

/// Split long text into multiple messages
pub fn split_posts(text: &str, limit: usize) -> Vec<String> {
    let mut posts = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
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

/// Main function generating Telegram posts from Markdown
pub fn generate_posts(mut input: String) -> Vec<String> {
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

    // Generate link to the full edition
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

    // Remove placeholder for full edition link
    input = input.replace("_Полный выпуск: ссылка_", "");

    let section_re = Regex::new(r"^##+\s+(.+)$").unwrap();
    let bullet_link_re = Regex::new(r"^[*-] ([^\[]+?)\((https?://[^\s\)]+)\)").unwrap();
    let star_bullet_link_re = Regex::new(r"^\* ([^\[]+?)\((https?://[^\s\)]+)\)").unwrap();
    let markdown_link_re = Regex::new(r"\[([^\]]+)\]\((https?://[^\s\)]+)\)").unwrap();
    let cotw_re = Regex::new(r"^\[(.+?)\]\((.+?)\)\s*[—-]\s*(.+)$").unwrap();

    let mut output = String::new();

    // Header
    if let Some(ref t) = title {
        output.push_str(&format!("**{}**", escape_markdown(t)));
    }
    if let Some(ref n) = number {
        output.push_str(&format!(" — \\#{}", escape_markdown(n)));
    }
    if let Some(ref d) = date {
        output.push_str(&format!(" — {}\n\n\\-\\-\\-\n", escape_markdown(d)));
    }

    let mut lines = input.lines().peekable();
    let mut current_section: Option<String> = None;
    let mut section_lines: Vec<String> = Vec::new();
    let mut first_section = true;
    let mut in_comment = false;

    while let Some(line) = lines.next() {
        if line.trim_start().starts_with("<!--") {
            in_comment = true;
        }
        if in_comment {
            if line.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        if let Some(sec) = section_re.captures(line) {
            // Finish previous section and output collected links
            if !section_lines.is_empty() {
                if !first_section {
                    output.push('\n');
                } else {
                    first_section = false;
                }
                if let Some(title) = current_section.take() {
                    output.push_str(&format!("**{}**\n", escape_markdown(&title)));
                    for l in &section_lines {
                        output.push_str(l);
                        output.push('\n');
                    }
                }
                section_lines.clear();
            }

            let title = sec[1].trim();
            // Crate of the Week processing
            if title == "Crate of the Week" {
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

        // Bullet links: - Text (url)
        if let Some(caps) = bullet_link_re.captures(line) {
            section_lines.push(format!(
                "\\- [{}]({})",
                escape_markdown(caps[1].trim()),
                escape_markdown_url(&caps[2])
            ));
            continue;
        }
        // Bullet links with asterisk: * Text (url)
        if let Some(caps) = star_bullet_link_re.captures(line) {
            section_lines.push(format!(
                "\\- [{}]({})",
                escape_markdown(caps[1].trim()),
                escape_markdown_url(&caps[2])
            ));
            continue;
        }

        // Markdown links outside bullet list
        if let Some(caps) = markdown_link_re.captures(line) {
            if !line.trim_start().starts_with('-') && !line.trim_start().starts_with('*') {
                section_lines.push(format!(
                    "[{}]({})",
                    escape_markdown(&caps[1]),
                    escape_markdown_url(&caps[2])
                ));
                continue;
            }
        }

        // Add remaining lines as is
        if !line.trim().is_empty() {
            section_lines.push(escape_markdown(line));
        }
    }

    // Output the last section if any
    if !section_lines.is_empty() {
        if !first_section {
            output.push('\n');
        }
        if let Some(title) = current_section.take() {
            output.push_str(&format!("**{}**\n", escape_markdown(&title)));
            for l in &section_lines {
                output.push_str(l);
                output.push('\n');
            }
        }
    }

    // Link to the full edition at the bottom as plain text
    output.push_str("\n\\-\\-\\-\n");
    if let Some(link) = url {
        output.push_str(&format!(
            "\nПолный выпуск: [{}]({})\n",
            escape_markdown(&link),
            escape_markdown_url(&link)
        ));
    }

    // Split into messages by Telegram limit
    let raw_posts = split_posts(&output, TELEGRAM_LIMIT);
    let total = raw_posts.len();
    raw_posts
        .into_iter()
        .enumerate()
        .map(|(i, post)| format!("*Часть {}/{}*\n{}", i + 1, total, post))
        .collect()
}

/// Write Telegram posts to files
pub fn write_posts(posts: &[String], dir: &Path) -> std::io::Result<()> {
    for (i, post) in posts.iter().enumerate() {
        let file_name = dir.join(format!("output_{}.md", i + 1));
        fs::write(&file_name, post)?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut input_path = "input.md";
    let mut plain = false;

    for arg in &args[1..] {
        if arg == "--plain" {
            plain = true;
        } else {
            input_path = arg;
        }
    }

    let input = fs::read_to_string(input_path)?;
    let mut posts = generate_posts(input);

    if plain {
        posts = posts.into_iter().map(|p| markdown_to_plain(&p)).collect();
    }

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

    #[test]
    fn plain_conversion() {
        let text = "*Часть 1/1*\n**News**\n\\- [Link](https://example.com)";
        let plain = markdown_to_plain(text);
        assert_eq!(plain, "Часть 1/1\nNews\n- Link (https://example.com)");
    }
}
