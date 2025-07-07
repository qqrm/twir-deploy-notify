use phf::phf_map;
use teloxide::utils::markdown::escape;

use crate::parser::{Section, parse_sections};
use crate::validator::validate_telegram_markdown;

pub const TELEGRAM_LIMIT: usize = 4000;

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
    format!("{e} **{}** {e}", escape_markdown(&upper), e = emoji)
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
            return format!(
                "**[{}]({})**",
                escape_markdown(text),
                escape_markdown_url(url)
            );
        }
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower == "quote of the week" {
        return format!("\n**{}:** 💬\n", escape_markdown(trimmed));
    }
    if let Some(emoji) = SUBHEADING_EMOJIS.get(lower.as_str()) {
        format!("\n**{}:** {}", escape_markdown(trimmed), emoji)
    } else {
        format!("**{}**", escape_markdown(trimmed))
    }
}

#[derive(Debug)]
pub struct ValidationError(pub String);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ValidationError {}

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
            sec.lines.insert(1, chat);
            sec.lines.insert(2, feed);
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
        header.push_str(&format!("\\#{}", escape_markdown(n)));
    }
    if let Some(ref d) = date {
        header.push_str(&format!(" — {}", escape_markdown(d)));
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
