use regex::Regex;
use std::{env, fs};

const TELEGRAM_LIMIT: usize = 4000;

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
    let input = fs::read_to_string(input_path)?;
    let mut output = String::new();

    // Title and date
    let title_re = Regex::new(r"(?m)^Title: (.+)$").unwrap();
    let number_re = Regex::new(r"(?m)^Number: (.+)$").unwrap();
    let date_re = Regex::new(r"(?m)^Date: (.+)$").unwrap();

    if let Some(title) = title_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!("**{}**", title.as_str()));
    }
    if let Some(number) = number_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!(" — #{}", number.as_str()));
    }
    if let Some(date) = date_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!(" — {}\n\n---\n\n", date.as_str()));
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
                    output.push_str(&format!("**{}**\n", title));
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
            section_links.push(format!("- [{}]({})", title, url));
        }
    }

    if !section_links.is_empty() {
        if !first_section {
            output.push('\n');
        }
        if let Some(title) = current_section.take() {
            output.push_str(&format!("**{}**\n", title));
            for link in &section_links {
                output.push_str(link);
                output.push('\n');
            }
        }
    }

    // Summary
    output.push_str("\n---\n\n");
    output.push_str("_Полный выпуск: ссылка_\n");

    let posts = split_posts(&output, TELEGRAM_LIMIT);

    for (i, post) in posts.iter().enumerate() {
        let file_name = format!("output_{}.md", i + 1);
        fs::write(&file_name, post)?;
        println!("Generated {}", file_name);
    }

    Ok(())
}
