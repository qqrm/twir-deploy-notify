use log::{debug, error, info};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{fs, path::Path, thread, time::Duration};
use teloxide::utils::markdown::{escape, escape_link_url};

use crate::parser::{Section, parse_sections};
use crate::validator::validate_telegram_markdown;

pub const TELEGRAM_LIMIT: usize = 4000;
pub const TELEGRAM_DELAY_MS: u64 = 1000;

fn replace_links(text: &str) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('[') {
        if let Some(mid) = rest[start..].find("](") {
            let mid = start + mid;
            if let Some(end) = rest[mid + 2..].find(')') {
                result.push_str(&rest[..start]);
                result.push_str(&rest[start + 1..mid]);
                result.push_str(" (");
                result.push_str(&rest[mid + 2..mid + 2 + end]);
                result.push(')');
                rest = &rest[mid + 2 + end + 1..];
                continue;
            }
        }
        break;
    }
    result.push_str(rest);
    result
}

fn find_value(text: &str, prefix: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn find_title(text: &str) -> Option<String> {
    find_value(text, "Title: ")
}

fn find_number(text: &str) -> Option<String> {
    find_value(text, "Number: ")
}

fn find_date(text: &str) -> Option<String> {
    find_value(text, "Date: ")
}

fn strip_header(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if !(line.starts_with("Title:") || line.starts_with("Number:") || line.starts_with("Date:"))
        {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Escape Telegram Markdown special characters in `text`.
///
/// # Parameters
/// - `text`: Plain text that may contain Markdown control characters.
///
/// # Returns
/// A new `String` with all reserved characters escaped so it can be used in
/// Telegram Markdown.
pub fn escape_markdown(text: &str) -> String {
    escape(text)
}

/// Escape parentheses in `url` for usage in Telegram Markdown links.
///
/// # Parameters
/// - `url`: The URL to escape.
///
/// # Returns
/// The escaped URL.
pub fn escape_markdown_url(url: &str) -> String {
    escape_link_url(url)
}

/// Format a section heading for Telegram posts.
///
/// # Parameters
/// - `title`: Raw section title text.
///
/// # Returns
/// A bold heading prefixed with a newspaper emoji.
pub fn format_heading(title: &str) -> String {
    let upper = title.to_uppercase();
    format!("📰 **{}**", escape_markdown(&upper))
}

/// Format a level 3 or level 4 heading.
///
/// # Parameters
/// - `title`: Heading text.
///
/// # Returns
/// The escaped heading wrapped in bold markers.
pub fn format_subheading(title: &str) -> String {
    format!("**{}**", escape_markdown(title))
}

/// Convert Telegram Markdown to a plain text representation.
///
/// # Parameters
/// - `text`: A Telegram Markdown string.
///
/// # Returns
/// A plain text version with formatting markers removed.
pub fn markdown_to_plain(text: &str) -> String {
    let without_escapes = text.replace('\\', "");
    let replaced = replace_links(&without_escapes);
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

/// Split a long message into chunks that obey Telegram's length limit.
///
/// # Parameters
/// - `text`: The text to split.
/// - `limit`: Maximum allowed length of each chunk.
///
/// # Returns
/// A vector of strings each no longer than `limit` characters.
pub fn split_posts(text: &str, limit: usize) -> Vec<String> {
    let mut posts = Vec::new();
    let mut current = String::new();
    let mut join_next = false;

    fn needs_escape(c: char) -> bool {
        matches!(c, '-' | '>' | '#' | '+' | '=' | '{' | '}' | '.' | '!')
    }

    for line in text.lines() {
        if line.len() > limit {
            if !current.is_empty() {
                posts.push(current.clone());
                current.clear();
            }

            let mut chunk = String::new();
            for c in line.chars() {
                if chunk.len() + c.len_utf8() > limit {
                    if chunk.ends_with('\\') {
                        chunk.pop();
                        posts.push(chunk.clone());
                        chunk.clear();
                        chunk.push('\\');
                    } else {
                        posts.push(chunk.clone());
                        chunk.clear();
                        if needs_escape(c) {
                            chunk.push('\\');
                        }
                    }
                }
                chunk.push(c);
            }
            if !chunk.is_empty() {
                posts.push(chunk);
            }
            join_next = false;
            continue;
        }

        let new_len = if current.is_empty() {
            line.len()
        } else if join_next {
            current.len() + line.len()
        } else {
            current.len() + 1 + line.len()
        };

        if new_len > limit && !current.is_empty() {
            if current.ends_with('\\') {
                current.pop();
                posts.push(current.clone());
                current.clear();
                current.push('\\');
                join_next = true;
            } else {
                posts.push(current.clone());
                current.clear();
                join_next = false;
            }
        }

        if !current.is_empty() && !join_next {
            current.push('\n');
        }
        if current.is_empty() {
            if let Some(first) = line.chars().next() {
                if needs_escape(first) && !line.starts_with('\\') {
                    current.push('\\');
                }
            }
        }
        current.push_str(line);
        if join_next {
            current.push('\n');
            join_next = false;
        }
    }

    if !current.is_empty() {
        posts.push(current);
    }

    posts
}

/// Convert a TWIR Markdown file into a series of Telegram posts.
///
/// The input may include metadata headers which will be used to build the
/// leading post and the final link section.
///
/// # Parameters
/// - `input`: Raw Markdown content read from a TWIR issue.
///
/// # Returns
/// A vector of validated Telegram Markdown posts or a `ValidationError` if any
/// post fails validation.
pub fn generate_posts(mut input: String) -> Result<Vec<String>, ValidationError> {
    let title = find_title(&input);
    let number = find_number(&input);
    let date = find_date(&input);

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

    let body = strip_header(&input);
    let mut sections = parse_sections(&body);

    if let Some(link) = url.as_ref() {
        let mut link_section = Section::default();
        link_section.lines.push("\\-\\-\\-".to_string());
        link_section.lines.push(String::new());
        link_section.lines.push(format!(
            "📖 Full issue: [{}]({})",
            escape_markdown(link),
            escape_markdown_url(link)
        ));
        sections.push(link_section);
    }

    let mut posts = Vec::new();

    let mut header = String::new();
    if let Some(ref t) = title {
        header.push_str(&format!("**{}**", escape_markdown(t)));
    }
    if let Some(ref n) = number {
        let already_in_title = title.as_ref().map(|t| t.contains(n)).unwrap_or(false);
        if !already_in_title {
            header.push_str(&format!(" — \\#{}", escape_markdown(n)));
        }
    }
    if let Some(ref d) = date {
        header.push_str(&format!(" — {}\n\n\\-\\-\\-\n", escape_markdown(d)));
    }

    for (idx, sec) in sections.iter().enumerate() {
        let mut section_text = String::new();
        if idx == 0 {
            section_text.push_str(&header);
        }
        if !sec.title.is_empty() {
            section_text.push_str(&format!("{}\n", format_heading(&sec.title)));
        }
        for line in &sec.lines {
            section_text.push_str(line);
            section_text.push('\n');
        }

        if section_text.len() > TELEGRAM_LIMIT {
            posts.extend(split_posts(&section_text, TELEGRAM_LIMIT));
        } else {
            posts.push(section_text);
        }
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
    let mut result = Vec::new();
    for (i, mut post) in final_posts.into_iter().enumerate() {
        if !post.ends_with('\n') {
            post.push('\n');
        }
        let formatted = format!("*Part {}/{}*\n{}", i + 1, total, post);
        validate_telegram_markdown(&formatted)
            .map_err(|e| ValidationError(format!("Generated post {} invalid: {e}", i + 1)))?;
        result.push(formatted);
    }
    Ok(result)
}

/// Write generated posts to `output_N.md` files in `dir`.
///
/// # Parameters
/// - `posts`: Messages to write.
/// - `dir`: Destination directory.
///
/// # Returns
/// `Ok(())` on success or any file I/O error encountered.
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

/// Send prepared posts to a Telegram chat via the HTTP API.
///
/// # Parameters
/// - `posts`: Posts to deliver.
/// - `base_url`: Base Telegram API endpoint.
/// - `token`: Bot token used for authentication.
/// - `chat_id`: Identifier of the destination chat or channel.
/// - `use_markdown`: Whether to enable Telegram Markdown parsing.
///
/// # Errors
/// Returns an error if the HTTP request fails or Telegram responds with an
/// error code.
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
            validate_telegram_markdown(post).map_err(ValidationError)?;
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
                "Telegram error for post {} {}: {}",
                i + 1,
                data.error_code.unwrap_or_default(),
                data.description.as_deref().unwrap_or("unknown")
            );
            return Err(format!(
                "Telegram API error in post {} {}: {}",
                i + 1,
                data.error_code.unwrap_or_default(),
                data.description.unwrap_or_default()
            )
            .into());
        }
        thread::sleep(Duration::from_millis(TELEGRAM_DELAY_MS));
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
        let posts = generate_posts(input.to_string()).unwrap();
        let mut dir = std::env::temp_dir();
        dir.push("twir_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        write_posts(&posts, &dir).unwrap();
        let first = dir.join("output_1.md");
        assert!(first.exists());
        let content = fs::read_to_string(first).unwrap();
        assert!(content.contains("*Part 1/2*"));
        assert!(content.contains("📰 **NEWS**"));
        assert!(content.contains("[Link](https://example.com)"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn plain_conversion() {
        let text = "*Part 1/1*\n**News**\n• [Link](https://example.com)";
        let plain = markdown_to_plain(text);
        assert_eq!(plain, "Part 1/1\nNews\n- Link (https://example.com)");
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
        let posts = generate_posts(input.to_string()).unwrap();
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
        let posts =
            generate_posts(format!("Title: T\nNumber: 1\nDate: 2025-01-01\n\n{text}")).unwrap();
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
    fn subheading_with_dash() {
        let text = "## Section\n### Foo-Bar\n- item\n";
        let secs = parse_sections(text);
        assert_eq!(secs[0].lines[0], "**Foo\\-Bar**");
        let posts = generate_posts(
            "Title: T\nNumber: 1\nDate: 2025-01-01\n\n## Section\n### Foo-Bar\n- item\n"
                .to_string(),
        )
        .unwrap();
        for p in posts {
            crate::validator::validate_telegram_markdown(&p).unwrap();
        }
    }

    #[test]
    fn dash_at_post_boundary() {
        let mut text = "a".repeat(10);
        text.push('\n');
        text.push_str("\\- start");
        let parts = split_posts(&text, 10);
        assert!(parts.len() > 1);
        assert!(parts[1].starts_with("\\-"));
        for p in parts {
            crate::validator::validate_telegram_markdown(&p).unwrap();
        }
    }

    #[test]
    fn backslash_then_dash_across_posts() {
        let mut text = "a".repeat(9);
        text.push('\\');
        text.push('\n');
        text.push_str("- test");
        let parts = split_posts(&text, 10);
        assert!(parts.len() > 1);
        assert!(parts[1].starts_with("\\-"));
        for p in parts {
            crate::validator::validate_telegram_markdown(&p).unwrap();
        }
    }

    #[test]
    fn markdown_validation() {
        assert!(crate::validator::validate_telegram_markdown("simple text").is_ok());
        assert!(crate::validator::validate_telegram_markdown("bad *text").is_err());
    }

    #[test]
    fn send_to_telegram_errors_on_invalid_markdown() {
        let posts = vec!["bad *text".to_string()];
        let err = send_to_telegram(&posts, "http://example.com", "TOKEN", "42", true);
        assert!(err.is_err());
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
                let posts = match generate_posts(input) {
                    Ok(p) => p,
                    Err(e) => return Err(TestCaseError::fail(e.to_string())),
                };
                prop_assert!(!posts.is_empty());
                for p in posts {
                    prop_assert!(p.len() <= TELEGRAM_LIMIT + 50);
                    if let Err(e) = crate::validator::validate_telegram_markdown(&p) {
                        return Err(TestCaseError::fail(e.to_string()));
                    }
                }
            }
        }
    }
}
