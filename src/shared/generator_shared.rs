use log::{debug, error, info, warn};
use phf::phf_map;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{borrow::Cow, fs, path::Path, thread, time::Duration};
use teloxide::utils::markdown::escape;

use crate::parser::{Section, parse_sections};
use crate::validator::validate_telegram_markdown;

pub const TELEGRAM_LIMIT: usize = 4000;
pub const TELEGRAM_DELAY_MS: u64 = 1000;
/// Delay before attempting to pin the first message.
pub const TELEGRAM_PIN_DELAY_MS: u64 = 2000;

/// Mapping of subheading titles to emojis used in Telegram posts.
pub static SUBHEADING_EMOJIS: phf::Map<&'static str, &'static str> = phf_map! {
    "newsletters" => "📰",
    "project/tooling updates" => "🛠️",
    "compiler" => "🛠️",
    "observations/thoughts" => "🤔",
    "rust walkthroughs" => "📚",
    "library" => "📚",
    "cargo" => "📦",
    "rustdoc" => "📖",
    "clippy" => "🔧",
    "rust-analyzer" => "🤖",
    "rust compiler performance triage" => "📊",
    "tracking issues & prs" => "📌",
};

/// Short URL guiding contributors how to submit CFP tasks.
const CFP_GUIDELINES: &str =
    "https://github.com/rust-lang/this-week-in-rust#call-for-participation-guidelines";

fn simplify_cfp_section(section: &mut Section) {
    let mut cleaned = Vec::new();
    let mut in_projects = false;
    let mut has_task = false;

    for line in section.lines.iter() {
        if line.starts_with("**CFP \\- Projects**") {
            in_projects = true;
            cleaned.push(line.clone());
            continue;
        }
        if line.starts_with("**CFP \\- Events**") {
            in_projects = false;
            cleaned.push(line.clone());
            continue;
        }
        if line.contains("guidelines") && line.contains("submit tasks") {
            continue;
        }
        if line.starts_with("Always wanted to contribute")
            || line.starts_with("Some of these tasks")
            || line.starts_with("Are you a new or experienced speaker")
        {
            continue;
        }
        if in_projects {
            if line.trim() == "No Calls for participation were submitted this week." {
                continue;
            }
            let trimmed = line.trim_start();
            if trimmed.starts_with('•') || trimmed.starts_with('*') {
                has_task = true;
            }
            cleaned.push(line.clone());
        } else {
            cleaned.push(line.clone());
        }
    }

    if !has_task {
        let msg = format!(
            "No new tasks this week\\. [Instructions]({})",
            escape_markdown_url(CFP_GUIDELINES)
        );
        cleaned.push(msg);
    }

    section.lines = cleaned;
}

fn simplify_quote_section(section: &mut Section) {
    let mut cleaned = Vec::new();
    let mut in_quote = false;
    for line in &section.lines {
        if line.contains("Quote of the Week") {
            cleaned.push(format_subheading("Quote of the Week"));
            in_quote = true;
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if in_quote {
            if lower.contains("please submit quotes")
                || lower.starts_with("thanks to")
                || lower.starts_with("despite ")
            {
                continue;
            }
            if line.trim_start().starts_with('–') {
                in_quote = false;
                cleaned.push(line.trim_start().to_string());
                cleaned.push(String::new());
                continue;
            }
            let content = line.trim_start_matches("\\>").trim_start();
            cleaned.push(format!("_{content}_"));
            continue;
        }
        cleaned.push(line.clone());
    }
    section.lines = cleaned;
}

fn simplify_jobs_section(section: &mut Section) {
    const PAT: &str = "Hiring thread on r/rust](";
    for line in &mut section.lines {
        if let Some(start) = line.find(PAT) {
            let url_start = start + PAT.len();
            if let Some(rest) = line.get(url_start..) {
                if let Some(end) = rest.find(')') {
                    let url = &rest[..end];
                    *line = format!("🦀 [Rust Job Reddit Thread]({})", escape_markdown_url(url));
                }
            }
        }
    }
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

/// Escape parentheses in `url` for usage in Telegram Markdown links.
///
/// # Parameters
/// - `url`: The URL to escape.
///
/// # Returns
/// The escaped URL.
pub fn escape_markdown_url(url: &str) -> String {
    url.replace('(', "\\(").replace(')', "\\)")
}

/// Format a section heading for Telegram posts.
///
/// # Parameters
/// - `title`: Raw section title text.
///
/// # Returns
/// A bold heading prefixed with an appropriate emoji.
pub fn format_heading(title: &str) -> String {
    let upper = title.to_uppercase();
    let emoji = match upper.as_str() {
        "UPCOMING EVENTS" => "🎉",
        "CRATE OF THE WEEK" => "📦",
        _ => "📰",
    };
    format!("{e} **{}** {e}", escape(&upper), e = emoji)
}

/// Format a level 3 or level 4 heading.
///
/// # Parameters
/// - `title`: Heading text.
///
/// # Returns
/// The escaped heading wrapped in bold markers.
pub fn format_subheading(title: &str) -> String {
    let trimmed = title.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(')') {
        if let Some(idx) = trimmed.find("](") {
            let text = &trimmed[1..idx];
            let url = &trimmed[idx + 2..trimmed.len() - 1];
            return format!("**[{}]({})**", escape(text), escape_markdown_url(url));
        }
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower == "quote of the week" {
        return format!("\n**{}:** 💬\n", escape(trimmed));
    }
    if let Some(emoji) = SUBHEADING_EMOJIS.get(lower.as_str()) {
        format!("\n**{}:** {}", escape(trimmed), emoji)
    } else {
        format!("**{}**", escape(trimmed))
    }
}

/// Convert Telegram Markdown to a plain text representation.
///
/// # Parameters
/// - `text`: A Telegram Markdown string.
///
/// # Returns
/// A plain text version with formatting markers removed.
pub fn markdown_to_plain(text: &str) -> String {
    use pulldown_cmark::{Event, Options, Parser, Tag};

    let without_escapes = text.replace('\\', "");
    let parser = Parser::new_ext(&without_escapes, Options::empty());
    let mut result = String::new();
    let mut in_link = false;
    let mut link_dest = String::new();
    let mut link_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Link(_, dest, _)) => {
                in_link = true;
                link_dest = dest.to_string();
                link_text.clear();
            }
            Event::End(Tag::Link(_, _, _)) => {
                result.push_str(&link_text.replace('•', "-"));
                result.push_str(" (");
                result.push_str(&link_dest);
                result.push(')');
                in_link = false;
            }
            Event::Text(t) | Event::Code(t) => {
                if in_link {
                    link_text.push_str(&t);
                } else {
                    result.push_str(&t.replace('•', "-"));
                }
            }
            Event::SoftBreak | Event::HardBreak => result.push('\n'),
            _ => {}
        }
    }

    if in_link {
        result.push_str(&link_text.replace('•', "-"));
        result.push_str(" (");
        result.push_str(&link_dest);
        result.push(')');
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
/// The function walks through the input line by line and builds a vector of
/// posts whose length never exceeds `limit`. Lines longer than the limit are
/// split character by character. When a chunk ends with a backslash the
/// character is moved to the beginning of the next chunk so that escape
/// sequences remain valid. If the start of a new post would begin with a
/// Markdown control character, it is prefixed with a backslash to keep the
/// formatting intact. Newlines are inserted between lines unless a trailing
/// backslash caused the next line to be joined.
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
    let mut in_code_block = false;

    fn needs_escape(c: char) -> bool {
        matches!(c, '-' | '>' | '#' | '+' | '=' | '{' | '}' | '.' | '!')
    }

    for line in text.lines() {
        if line.trim() == "```" {
            let extra = if current.is_empty() { 3 } else { 4 };
            if current.len() + extra > limit && !current.is_empty() {
                if in_code_block {
                    current.push_str("\n```");
                    posts.push(current.clone());
                    current.clear();
                    current.push_str("```");
                } else {
                    posts.push(current.clone());
                    current.clear();
                }
            } else if !current.is_empty() {
                current.push('\n');
            }
            current.push_str("```");
            in_code_block = !in_code_block;
            continue;
        }
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
            if in_code_block {
                current.push_str("\n```");
                posts.push(current.clone());
                current.clear();
                current.push_str("```");
            } else if current.ends_with('\\') {
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
    let _title = find_title(&input);
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
    for sec in &mut sections {
        if sec.title.eq_ignore_ascii_case("Jobs") && !sec.lines.is_empty() {
            simplify_jobs_section(sec);
            let chat = format!(
                "💼 [Rust Jobs chat]({})",
                escape_markdown_url("https://t.me/rust_jobs")
            );
            let feed = format!(
                "📢 [Rust Jobs feed]({})",
                escape_markdown_url("https://t.me/rust_jobs_feed")
            );
            let hh = format!(
                "📝 [Rust HH jobs]({}) — channel with Rust jobs from HeadHunter",
                escape_markdown_url("https://t.me/rusthhjobs")
            );
            sec.lines.insert(1, chat);
            sec.lines.insert(2, feed);
            sec.lines.insert(3, hh);
        }
        if sec
            .title
            .eq_ignore_ascii_case("Call for Participation; projects and speakers")
        {
            simplify_cfp_section(sec);
        }
        simplify_quote_section(sec);
    }
    sections.retain(|s| !s.title.eq_ignore_ascii_case("Upcoming Events"));

    if let Some(link) = url.as_ref() {
        let mut link_section = Section::default();
        link_section.lines.push(String::new());
        link_section.lines.push(format!(
            "🌐 [View web version]({}) 🌐",
            escape_markdown_url(link)
        ));
        sections.push(link_section);
    }

    let mut posts = Vec::new();

    let mut header = String::new();
    if let Some(ref n) = number {
        header.push_str(&format!("\\#{}", escape(n)));
    }
    if let Some(ref d) = date {
        header.push_str(&format!(" — {}", escape(d)));
    }
    if !header.is_empty() {
        header.push_str("\n\n");
    }

    let mut current_post = String::new();

    for (idx, sec) in sections.iter().enumerate() {
        let mut section_text = String::new();
        if idx == 0 {
            section_text.push_str(&header);
        }
        if !sec.title.is_empty() {
            if idx > 0 {
                section_text.push('\n');
            }
            section_text.push_str(&format!("{}\n", format_heading(&sec.title)));
        }
        for line in &sec.lines {
            section_text.push_str(line);
            section_text.push('\n');
        }

        if !current_post.is_empty() && current_post.len() + section_text.len() > TELEGRAM_LIMIT {
            posts.push(current_post);
            current_post = section_text;
        } else {
            current_post.push_str(&section_text);
        }
    }

    if !current_post.is_empty() {
        posts.push(current_post);
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
        let formatted = if i == 0 {
            post.trim_start_matches('\n').to_string()
        } else {
            format!(
                "*Part {}/{}*\n\n{}",
                i + 1,
                total,
                post.trim_start_matches('\n')
            )
        };
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
    fs::create_dir_all(dir)?;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("output_") && name.ends_with(".md") {
                    fs::remove_file(entry.path())?;
                }
            }
        }
    }
    for (i, post) in posts.iter().enumerate() {
        let file_name = dir.join(format!("output_{}.md", i + 1));
        fs::write(&file_name, post)?;
        if let Some(name) = file_name.file_name() {
            println!("Generated {}", name.to_string_lossy());
        }
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

/// Normalize chat identifier to the `-100` prefix used for channels.
pub fn normalize_chat_id(chat_id: &str) -> Cow<'_, str> {
    if chat_id.starts_with('@') || chat_id.starts_with("-100") {
        Cow::Borrowed(chat_id)
    } else {
        Cow::Owned(format!("-100{}", chat_id.trim_start_matches('-')))
    }
}

/// Replace the bot token in `url` with "<token>" for logging purposes.
///
/// # Parameters
/// - `url`: The full Telegram API URL.
/// - `token`: The bot token to redact.
///
/// # Returns
/// The URL with the token substituted by the placeholder.
fn sanitize_url(url: &str, token: &str) -> String {
    url.replace(token, "<token>")
}

/// Send prepared posts to a Telegram chat via the HTTP API.
///
/// # Parameters
/// - `posts`: Posts to deliver.
/// - `base_url`: Base Telegram API endpoint.
/// - `token`: Bot token used for authentication.
/// - `chat_id`: Identifier of the destination chat or channel.
/// - `use_markdown`: Whether to enable Telegram Markdown parsing.
/// - `pin_first`: Pin the first sent message using `pinChatMessage`.
///   The pin request and deletion of the service message are performed
///   *after* all posts have been successfully sent.
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
    pin_first: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if use_markdown {
        for (i, post) in posts.iter().enumerate() {
            validate_telegram_markdown(post)
                .map_err(|e| ValidationError(format!("Post {} invalid: {e}", i + 1)))?;
        }
    }

    let client = Client::new();
    let chat_id = normalize_chat_id(chat_id);
    info!("Sending {} posts", posts.len());
    let mut first_id: Option<i64> = None;
    let mut last_id: Option<i64> = None;
    for (i, post) in posts.iter().enumerate() {
        info!("Posting {}/{} ({} chars)", i + 1, posts.len(), post.len());
        let url = format!(
            "{}/bot{}/sendMessage",
            base_url.trim_end_matches('/'),
            token
        );
        let safe_url = sanitize_url(&url, token);
        debug!("Posting message {} via {safe_url}", i + 1);
        let mut form = vec![("chat_id", chat_id.as_ref()), ("text", post)];
        if use_markdown {
            form.push(("parse_mode", "MarkdownV2"));
        }
        form.push(("disable_web_page_preview", "true"));

        let resp = client.post(&url).form(&form).send()?;
        let status = resp.status();
        let body = resp.text()?;
        debug!("Telegram response {status}: {body}");
        let raw: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse Telegram response: {e}: {body}"))?;
        let ok = raw.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        if !ok {
            let code = raw
                .get("error_code")
                .and_then(|v| v.as_i64())
                .unwrap_or_default();
            let desc = raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            error!(
                "Telegram error for post {} to {} via {} {}: {} (status {})",
                i + 1,
                chat_id,
                safe_url,
                code,
                desc,
                status.as_u16()
            );
            error!("Full response: {body}");
            let snippet: String = post.chars().take(100).collect();
            error!("Post snippet: {snippet}");
            return Err(format!("Telegram API error in post {} {}: {}", i + 1, code, desc).into());
        }
        info!("Post {} sent", i + 1);
        if pin_first {
            match raw
                .get("result")
                .and_then(|v| v.get("message_id"))
                .and_then(|v| v.as_i64())
            {
                Some(id) => {
                    debug!("Received message_id {id}");
                    if i == 0 {
                        first_id = Some(id);
                    }
                    last_id = Some(id);
                }
                None if i == 0 => {
                    return Err("Telegram response missing message_id".into());
                }
                None => {}
            }
        }
        if i + 1 < posts.len() {
            thread::sleep(Duration::from_millis(TELEGRAM_DELAY_MS));
        }
    }
    if let Some(msg_id) = first_id {
        debug!("Sleeping {TELEGRAM_PIN_DELAY_MS} ms before pinning");
        thread::sleep(Duration::from_millis(TELEGRAM_PIN_DELAY_MS));
        let pin_url = format!(
            "{}/bot{}/pinChatMessage",
            base_url.trim_end_matches('/'),
            token
        );
        debug!("Pinning message {msg_id} via /pinChatMessage");
        let msg_id_str = msg_id.to_string();
        let pin_form = vec![("chat_id", chat_id.as_ref()), ("message_id", &msg_id_str)];
        let resp = client.post(&pin_url).form(&pin_form).send()?;
        let status = resp.status();
        let body = resp.text()?;
        debug!("Telegram pin response {status}: {body}");
        let pin_data: TelegramResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse Telegram pin response: {e}: {body}"))?;
        if !pin_data.ok {
            error!(
                "Telegram error pinning message {} {}: {}",
                msg_id,
                pin_data.error_code.unwrap_or_default(),
                pin_data.description.as_deref().unwrap_or("unknown")
            );
            return Err(format!(
                "Telegram API error when pinning {} {}: {}",
                msg_id,
                pin_data.error_code.unwrap_or_default(),
                pin_data.description.unwrap_or_default()
            )
            .into());
        }

        // Attempt to remove the service message about the pinned post.
        let delete_url = format!(
            "{}/bot{}/deleteMessage",
            base_url.trim_end_matches('/'),
            token
        );
        let notif_id = match last_id {
            Some(id) => id + 1,
            None => msg_id + 1,
        };
        let notif_id = notif_id.to_string();
        let delete_form = vec![("chat_id", chat_id.as_ref()), ("message_id", &notif_id)];
        let resp = client.post(&delete_url).form(&delete_form).send()?;
        let status = resp.status();
        let body = resp.text()?;
        debug!("Telegram delete response {status}: {body}");
        let delete_data: TelegramResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse Telegram delete response: {e}: {body}"))?;
        if !delete_data.ok {
            warn!(
                "Telegram error deleting pin notification {} {}: {}",
                notif_id,
                delete_data.error_code.unwrap_or_default(),
                delete_data.description.as_deref().unwrap_or("unknown")
            );
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
        let posts = generate_posts(input.to_string()).unwrap();
        let mut dir = std::env::temp_dir();
        dir.push("twir_test");
        let _ = fs::remove_dir_all(&dir);
        write_posts(&posts, &dir).unwrap();
        let first = dir.join("output_1.md");
        assert!(first.exists());
        let content = fs::read_to_string(first).unwrap();
        assert!(!content.starts_with("*Part"));
        assert!(content.starts_with("\\#1 — 2024\\-01\\-01"));
        assert!(content.contains("📰 **NEWS** 📰"));
        assert!(content.contains("[Link](https://example.com)"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn old_output_files_are_removed() {
        let dir = tempfile::tempdir().unwrap();
        let output1 = dir.path().join("output_1.md");
        let output2 = dir.path().join("output_2.md");
        fs::write(&output1, "old").unwrap();
        fs::write(&output2, "old").unwrap();
        let posts = vec!["new".to_string()];
        write_posts(&posts, dir.path()).unwrap();
        assert!(output1.exists());
        assert!(!output2.exists());
        let content = fs::read_to_string(output1).unwrap();
        assert_eq!(content, "new");
    }

    #[test]
    fn plain_conversion() {
        let text = "*Part 2/3*\n**News**\n• [Link](https://example.com)";
        let plain = markdown_to_plain(text);
        assert_eq!(plain, "Part 2/3\nNews\n- Link (https://example.com)");
    }

    #[test]
    fn plain_code_block() {
        let text = "```rust\nlet x = 1;\n```";
        let plain = markdown_to_plain(text);
        assert_eq!(plain, "let x = 1;\n");
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
        let escaped = escape(text);
        assert_eq!(escaped, "\\_bold\\_ \\*italic\\*");
    }

    #[test]
    fn escape_url_parentheses() {
        let url = "https://example.com/path(1)";
        let escaped = escape_markdown_url(url);
        assert_eq!(escaped, "https://example.com/path\\(1\\)");
    }

    #[test]
    fn table_rendering() {
        let input = "Title: Test\nNumber: 1\nDate: 2024-01-01\n\n## Table\n| Name | Score |\n|------|------|\n| Foo | 10 |\n| Bar | 20 |\n";
        let posts = generate_posts(input.to_string()).unwrap();
        let table = "```\n| Name | Score |\n| Foo  | 10    |\n| Bar  | 20    |\n```";
        assert!(posts[0].contains(table));
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
        assert_eq!(formatted, "📰 **MY TITLE** 📰");
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
        let err = send_to_telegram(&posts, "http://example.com", "TOKEN", "42", true, false);
        assert!(err.is_err());
    }

    #[test]
    fn cfp_sections_without_content_are_short() {
        let input = include_str!("../../tests/2025-07-05-call-for-participation.md");
        let posts = generate_posts(input.to_string()).unwrap();
        assert_eq!(posts.len(), 1);
        let post = &posts[0];
        assert!(post.contains("No Calls for participation were submitted this week"));
        assert!(post.contains("No Calls for papers or presentations were submitted this week"));
        assert!(!post.contains("Always wanted to contribute"));
        assert!(!post.contains("Some of these tasks"));
        assert!(!post.contains("Are you a new or experienced speaker"));
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
