use regex::Regex;
use std::{env, fs};

const TELEGRAM_LIMIT: usize = 4000;

/// Escape characters that have special meaning in Telegram MarkdownV2.
fn escape_markdown(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '_' | '*' | '[' | ']' | '(' | ')' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn split_posts(text: &str, limit: usize) -> Vec<String> {
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

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).map(String::as_str).unwrap_or("input.md");
    let mut input = fs::read_to_string(input_path)?;
    let mut output = String::new();

    // Title and date
    let title_re = Regex::new(r"(?m)^Title: (.+)$").unwrap();
    let number_re = Regex::new(r"(?m)^Number: (.+)$").unwrap();
    let date_re = Regex::new(r"(?m)^Date: (.+)$").unwrap();

    if let Some(title) = title_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!("**{}**", escape_markdown(title.as_str())));
    }
    
    if let Some(number) = number_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!(" — #{}", escape_markdown(number.as_str())));
    }
    
    if let Some(date) = date_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!(" — {}\n\n---\n\n", escape_markdown(date.as_str())));
    }
  
    let date = date_re
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    if let Some(ref d) = date {
        output.push_str(&format!(" — {}\n\n---\n\n", d));
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
        input = input.replace("_Полный выпуск: ссылка_", &format!("_Полный выпуск: {}_", link));
    }

    // Sections and links
    let section_re = Regex::new(r"^##+\s+(.+)$").unwrap();
    let link_re = Regex::new(r"^[*-] \[(.+?)\]\((.+?)\)").unwrap();
    let cotw_re = Regex::new(r"^\[(.+?)\]\((.+?)\)\s*[—-]\s*(.+)$").unwrap();

    let mut lines = input.lines().peekable();
    // Current section name and collected links
    let mut current_section: Option<String> = None;
    let mut section_links: Vec<String> = Vec::new();
    let mut first_section = true;

    while let Some(line) = lines.next() {
        if let Some(sec) = section_re.captures(line) {
            if !section_links.is_empty() {
                if !first_section {
                    output.push('\n');
                } else {
                    first_section = false;
                }
                if let Some(title) = current_section.take() {
                    output.push_str(&format!("**{}**\n", escape_markdown(&title)));
                    for link in &section_links {
                        output.push_str(link);
                        output.push('\n');
                    }
                }
                section_links.clear();
            }

            let title = sec[1].trim();
            if title == "Crate of the Week" {
                if let Some(next_line) = lines.next() {
                    if let Some(caps) = cotw_re.captures(next_line) {
                        if !first_section {
                            output.push('\n');
                        } else {
                            first_section = false;
                        }
                        output.push_str("**Crate of the Week**\n");
                        output.push_str(&format!("- [{}]({}) — {}\n", &caps[1], &caps[2], &caps[3]));
                    }
                }
                current_section = None;
            } else {
                current_section = Some(title.to_string());
            }
            continue;
        }

        if let Some(caps) = link_re.captures(line) {
            let title = &caps[1];
            let url = &caps[2];
            section_links.push(format!(
                "- [{}]({})",
                escape_markdown(title),
                escape_markdown(url)
            ));
        }
    }

    if !section_links.is_empty() {
        if !first_section {
            output.push('\n');
        }
        if let Some(title) = current_section.take() {
            output.push_str(&format!("**{}**\n", escape_markdown(&title)));
            for link in &section_links {
                output.push_str(link);
                output.push('\n');
            }
        }
    }

    // Summary
    output.push_str("\n---\n\n");
    if let Some(link) = url {
        output.push_str(&format!("_Полный выпуск: {}_\n", link));
    }

    let posts = split_posts(&output, TELEGRAM_LIMIT);
    let total = posts.len();

    // Add part number prefix to each fragment
    for (i, post) in posts.iter().enumerate() {
        let numbered = format!("*Часть {}/{}*\n{}", i + 1, total, post);
        let file_name = format!("output_{}.md", i + 1);
        fs::write(&file_name, numbered)?;
        println!("Generated {}", file_name);
    }

    Ok(())
}
