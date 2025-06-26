use regex::Regex;
use std::{env, fs};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).map(String::as_str).unwrap_or("input.md");
    let input = fs::read_to_string(input_path)?;
    let mut output = String::new();

    // Заголовок и дата
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
    output.push_str("_Полный выпуск: ссылка_\n");

    fs::write("output.md", &output)?;
    println!("{}", output);

    Ok(())
}
