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

    // Заголовок и дата
    let title_re = Regex::new(r"(?m)^Title: (.+)$").unwrap();
    let number_re = Regex::new(r"(?m)^Number: (.+)$").unwrap();
    let date_re = Regex::new(r"(?m)^Date: (.+)$").unwrap();
    let url_re = Regex::new(r"(?mi)^URL: (.+)$").unwrap();

    if let Some(title) = title_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!("**{}**", title.as_str()));
    }
    if let Some(number) = number_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!(" — #{}", number.as_str()));
    }
    if let Some(date) = date_re.captures(&input).and_then(|c| c.get(1)) {
        output.push_str(&format!(" — {}\n\n---\n\n", date.as_str()));
    }

    let url = url_re
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());

    // Разделы и ссылки
    let section_re = Regex::new(r"^##+\s+(.+)$").unwrap();
    let link_re = Regex::new(r"^[*-] \[(.+?)\]\((.+?)\)").unwrap();

    let mut lines = input.lines().peekable();
    let mut in_section = false;
    let mut current_section = String::new();

    while let Some(line) = lines.next() {
        if let Some(sec) = section_re.captures(line) {
            if in_section {
                output.push('\n');
            }
            current_section = sec[1].trim().to_string();
            output.push_str(&format!("**{}**\n", current_section));
            in_section = true;
            continue;
        }

        if in_section {
            if let Some(caps) = link_re.captures(line) {
                let title = &caps[1];
                let url = &caps[2];
                output.push_str(&format!("- [{}]({})\n", title, url));
            }
        }
    }

    // Итог
    output.push_str("\n---\n\n");
    if let Some(link) = url {
        output.push_str(&format!("_Полный выпуск: [{0}]({0})_\n", link));
    }

    let posts = split_posts(&output, TELEGRAM_LIMIT);

    for (i, post) in posts.iter().enumerate() {
        let file_name = format!("output_{}.md", i + 1);
        fs::write(&file_name, post)?;
        println!("Generated {}", file_name);
    }

    Ok(())
}
