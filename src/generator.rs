use log::{debug, error, info};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{fs, path::Path};
use teloxide::utils::markdown::{escape, escape_link_url};

use crate::parser::{Section, parse_sections};

pub const TELEGRAM_LIMIT: usize = 4000;

static LINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());
static TITLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^Title: (.+)$").unwrap());
static NUMBER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^Number: (.+)$").unwrap());
static DATE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^Date: (.+)$").unwrap());
static HEADER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^(Title|Number|Date):.*$\n?").unwrap());

pub fn escape_markdown(text: &str) -> String {
    escape(text)
}

pub fn escape_markdown_url(url: &str) -> String {
    escape_link_url(url)
}

pub fn format_heading(title: &str) -> String {
    let upper = title.to_uppercase();
    format!("📰 **{}**", escape_markdown(&upper))
}

pub fn format_subheading(title: &str) -> String {
    format!("**{}**", escape_markdown(title))
}

pub fn markdown_to_plain(text: &str) -> String {
    let without_escapes = text.replace('\\', "");
    let replaced = LINK_RE.replace_all(&without_escapes, "$1 ($2)");
    let mut result = String::with_capacity(replaced.len());
    for (i, line) in replaced.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let mut line_no_format = line.replace('*', "");
        line_no_format = line_no_format.replace('•', "-");
        result.push_str(&line_no_format);
    }
    result
}

#[derive(Debug)]
pub struct ValidationError(pub String);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_telegram_markdown(text: &str) -> Result<(), ValidationError> {
    use tbot::markup::markdown_v2::ESCAPED_TEXT_CHARACTERS;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next();
            continue;
        }
        if ESCAPED_TEXT_CHARACTERS.contains(&c) {
            return Err(ValidationError(format!(
                "unescaped markdown character '{c}'"
            )));
        }
    }
    Ok(())
}

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

pub fn generate_posts(mut input: String) -> Vec<String> {
    let title = TITLE_RE
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    let number = NUMBER_RE
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    let date = DATE_RE
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());

    let url = if let (Some(d), Some(n)) = (date.as_ref(), number.as_ref()) {
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

    input = input.replace("_Полный выпуск: ссылка_", "");

    let body = HEADER_RE.replace_all(&input, "");
    let mut sections = parse_sections(&body);

    if let Some(link) = url.as_ref() {
        let mut link_section = Section::default();
        link_section.lines.push("\\-\\-\\-".to_string());
        link_section.lines.push(String::new());
        link_section.lines.push(format!(
            "Полный выпуск: [{}]({})",
            escape_markdown(link),
            escape_markdown_url(link)
        ));
        sections.push(link_section);
    }

    let mut posts = Vec::new();
    let mut current = String::new();

    if let Some(ref t) = title {
        current.push_str(&format!("**{}**", escape_markdown(t)));
    }
    if let Some(ref n) = number {
        current.push_str(&format!(" — \\#{}", escape_markdown(n)));
    }
    if let Some(ref d) = date {
        current.push_str(&format!(" — {}\n\n\\-\\-\\-\n", escape_markdown(d)));
    }

    for sec in &sections {
        let mut section_text = String::new();
        if !sec.title.is_empty() {
            section_text.push_str(&format!("{}\n", format_heading(&sec.title)));
        }
        for line in &sec.lines {
            section_text.push_str(line);
            section_text.push('\n');
        }

        if current.len() + section_text.len() > TELEGRAM_LIMIT && !current.is_empty() {
            posts.push(current.clone());
            current.clear();
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(&section_text);
    }

    if !current.is_empty() {
        posts.push(current);
    }

    let mut final_posts = Vec::new();
    for post in posts {
        if post.len() > TELEGRAM_LIMIT {
            final_posts.extend(split_posts(&post, TELEGRAM_LIMIT));
        } else {
            final_posts.push(post);
        }
    }

    let total = final_posts.len();
    final_posts
        .into_iter()
        .enumerate()
        .map(|(i, mut post)| {
            if !post.ends_with('\n') {
                post.push('\n');
            }
            format!("*Часть {}/{}*\n{}", i + 1, total, post)
        })
        .collect()
}

pub fn write_posts(posts: &[String], dir: &Path) -> std::io::Result<()> {
    for (i, post) in posts.iter().enumerate() {
        let file_name = dir.join(format!("output_{}.md", i + 1));
        fs::write(&file_name, post)?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct TelegramResponse {
    ok: bool,
    #[serde(default)]
    error_code: Option<i32>,
    #[serde(default)]
    description: Option<String>,
}

pub fn send_to_telegram(
    posts: &[String],
    base_url: &str,
    token: &str,
    chat_id: &str,
    use_markdown: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    info!("Sending {} posts", posts.len());
    for (i, post) in posts.iter().enumerate() {
        let url = format!(
            "{}/bot{}/sendMessage",
            base_url.trim_end_matches('/'),
            token
        );
        debug!("Posting message {} to {}", i + 1, url);
        let mut form = vec![("chat_id", chat_id), ("text", post)];
        if use_markdown {
            validate_telegram_markdown(post)?;
            form.push(("parse_mode", "MarkdownV2"));
        }
        form.push(("disable_web_page_preview", "true"));

        let resp = client.post(&url).form(&form).send()?;
        let status = resp.status();
        let body = resp.text()?;
        debug!("Telegram response {status}: {body}");
        let data: TelegramResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse Telegram response: {e}: {body}"))?;
        if !data.ok {
            error!(
                "Telegram error {}: {}",
                data.error_code.unwrap_or_default(),
                data.description.as_deref().unwrap_or("unknown")
            );
            return Err(format!(
                "Telegram API error {}: {}",
                data.error_code.unwrap_or_default(),
                data.description.unwrap_or_default()
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_sections;
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
        assert!(content.contains("📰 **NEWS**"));
        assert!(content.contains("[Link](https://example.com)"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn plain_conversion() {
        let text = "*Часть 1/1*\n**News**\n• [Link](https://example.com)";
        let plain = markdown_to_plain(text);
        assert_eq!(plain, "Часть 1/1\nNews\n- Link (https://example.com)");
    }

    #[test]
    fn link_parsing() {
        let text = "## Links\n- [Rust](https://rust-lang.org)\n";
        let secs = parse_sections(text);
        assert_eq!(secs.len(), 1);
        assert_eq!(secs[0].title, "Links");
        assert_eq!(secs[0].lines, vec!["• [Rust](https://rust-lang.org)"]);
    }

    #[test]
    fn nested_list_parsing() {
        let text = "## News\n- Item 1\n  - Sub 1\n  - Sub 2\n- Item 2\n";
        let secs = parse_sections(text);
        assert_eq!(secs.len(), 1);
        assert_eq!(secs[0].title, "News");
        assert_eq!(
            secs[0].lines,
            vec!["• Item 1", "  • Sub 1", "  • Sub 2", "• Item 2",]
        );
    }

    #[test]
    fn escape_markdown_basic() {
        let text = "_bold_ *italic*";
        let escaped = escape_markdown(text);
        assert_eq!(escaped, "\\_bold\\_ \\*italic\\*");
    }

    #[test]
    fn escape_url_parentheses() {
        let url = "https://example.com/path(1)";
        let escaped = escape_markdown_url(url);
        assert_eq!(escaped, "https://example.com/path(1\\)");
    }

    #[test]
    fn table_rendering() {
        let input = "Title: Test\nNumber: 1\nDate: 2024-01-01\n\n## Table\n| Name | Score |\n|------|------|\n| Foo | 10 |\n| Bar | 20 |\n";
        let posts = generate_posts(input.to_string());
        assert!(posts[0].contains("| Name | Score |"));
        assert!(posts[0].contains("| Foo  | 10    |"));
        assert!(posts[0].contains("| Bar  | 20    |"));
    }

    #[test]
    fn quote_and_code_blocks() {
        let text = "## Test\n> quoted text\n\n```\ncode line\n```\n";
        let secs = parse_sections(text);
        assert_eq!(secs.len(), 1);
        assert_eq!(secs[0].title, "Test");
        assert_eq!(
            secs[0].lines,
            vec!["\\> quoted text", "```\ncode line\n```"]
        );
        let posts = generate_posts(format!("Title: T\nNumber: 1\nDate: 2025-01-01\n\n{text}"));
        let combined = posts.join("\n");
        assert!(combined.contains("> quoted text"));
        assert!(combined.contains("```\ncode line\n```"));
    }

    #[test]
    fn bullet_formatting() {
        let text = "## Items\n- example\n";
        let secs = parse_sections(text);
        assert_eq!(secs[0].lines, vec!["• example"]);
        let plain = markdown_to_plain(&secs[0].lines[0]);
        assert!(plain.starts_with("- "));
    }

    #[test]
    fn heading_formatter() {
        let formatted = format_heading("My Title");
        assert_eq!(formatted, "📰 **MY TITLE**");
    }

    #[test]
    fn markdown_validation() {
        assert!(validate_telegram_markdown("simple text").is_ok());
        assert!(validate_telegram_markdown("bad *text").is_err());
    }

    mod property {
        use super::*;
        use proptest::prelude::*;

        fn arb_heading() -> impl Strategy<Value = String> {
            "[A-Za-z0-9 ]{1,20}".prop_map(|s| format!("## {s}"))
        }

        fn arb_list() -> impl Strategy<Value = String> {
            prop::collection::vec("[A-Za-z0-9 ]{1,20}", 1..5).prop_map(|items| {
                items
                    .into_iter()
                    .map(|s| format!("- {s}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }

        fn arb_table() -> impl Strategy<Value = String> {
            prop::collection::vec(("[A-Za-z0-9 ]{1,10}", "[A-Za-z0-9 ]{1,10}"), 1..4).prop_map(
                |rows| {
                    let mut table = String::from("| Col1 | Col2 |\n|------|------|\n");
                    for (c1, c2) in rows {
                        table.push_str(&format!("| {c1} | {c2} |\n"));
                    }
                    table
                },
            )
        }

        fn arb_body() -> impl Strategy<Value = String> {
            prop::collection::vec(prop_oneof![arb_heading(), arb_list(), arb_table()], 1..8)
                .prop_map(|parts| parts.join("\n"))
        }

        fn arb_markdown() -> impl Strategy<Value = String> {
            arb_body()
                .prop_map(|body| format!("Title: Test\nNumber: 1\nDate: 2025-01-01\n\n{body}"))
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(32))]

            #[test]
            fn random_inputs_are_split_correctly(input in arb_markdown()) {
                let posts = generate_posts(input);
                prop_assert!(!posts.is_empty());
                for p in posts {
                    prop_assert!(p.len() <= TELEGRAM_LIMIT + 50);
                }
            }
        }
    }
}
