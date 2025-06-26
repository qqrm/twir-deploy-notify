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
    let mut input = fs::read_to_string(input_path)?;
    let mut output = String::new();

    // Заголовок и дата
    let title_re = Regex::new(r"(?m)^Title: (.+)$").unwrap();
    let number_re = Regex::new(r"(?m)^Number: (.+)$").unwrap();
    let date_re = Regex::new(r"(?m)^Date: (.+)$").unwrap();

    if let Some(title) = title_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!("**{}**", title.as_str()));
    }
    let number = number_re
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    if let Some(ref num) = number {
        output.push_str(&format!(" — #{}", num));
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

    // Разделы и ссылки
    let section_re = Regex::new(r"^##+\s+(.+)$").unwrap();
    let link_re = Regex::new(r"^[*-] \[(.+?)\]\((.+?)\)").unwrap();

    let mut lines = input.lines().peekable();
    // Название текущего раздела и накопленные ссылки
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
            current_section = Some(sec[1].trim().to_string());
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

    // Итог
    output.push_str("\n---\n\n");
    if let Some(link) = url {
        output.push_str(&format!("_Полный выпуск: {}_\n", link));
    }

    let posts = split_posts(&output, TELEGRAM_LIMIT);

    for (i, post) in posts.iter().enumerate() {
        let file_name = format!("output_{}.md", i + 1);
        fs::write(&file_name, post)?;
        println!("Generated {}", file_name);
    }

    Ok(())
}
