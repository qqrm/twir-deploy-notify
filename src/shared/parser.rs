use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::generator::{escape_markdown_url, format_subheading};
use teloxide::utils::markdown::escape;

/// Representation of a single TWIR section.
#[derive(Default)]
pub struct Section {
    pub title: String,
    pub lines: Vec<String>,
}

fn fix_bare_link(line: &str) -> String {
    if line.contains("](") {
        return line.to_string();
    }
    let plain = line.replace('\\', "");
    let trimmed = plain.trim_end();
    if trimmed.ends_with(')')
        && let Some(start) = trimmed
            .rfind("(https://")
            .or_else(|| trimmed.rfind("(http://"))
    {
        let url = &trimmed[start + 1..trimmed.len() - 1];
        let text = trimmed[..start].trim_end();
        return format!("[{}]({})", escape(text), escape_markdown_url(url));
    }
    line.to_string()
}

fn replace_github_mentions(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut bracket_depth = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '[' => {
                bracket_depth += 1;
                result.push(ch);
                i += 1;
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                result.push(ch);
                i += 1;
            }
            '@' if bracket_depth == 0 => {
                let prev = if i == 0 { None } else { Some(chars[i - 1]) };
                let mut j = i + 1;
                if j < chars.len() && chars[j].is_ascii_alphanumeric() {
                    while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '-') {
                        j += 1;
                    }
                    let next = chars.get(j).copied();
                    if !prev.is_some_and(|c| c.is_ascii_alphanumeric() || c == '/')
                        && !next.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-')
                    {
                        let user: String = chars[i + 1..j].iter().collect();
                        result.push_str(&format!("[{user}](https://github.com/{user})"));
                        i = j;
                        continue;
                    }
                }
                result.push('@');
                i += 1;
            }
            _ => {
                result.push(ch);
                i += 1;
            }
        }
    }
    result
}

fn normalize_table_text(text: &str) -> String {
    text.replace("Regressions", "Reg")
        .replace("Improvements", "Imp")
        .replace('❌', "x")
        .replace('✅', "v")
        .replace("(primary)", "prim")
        .replace("(secondary)", "sec")
        .replace("primary", "prim")
        .replace("secondary", "sec")
}

/// Parse TWIR Markdown into logical sections using `pulldown-cmark`.
///
/// # Parameters
/// - `text`: Full Markdown source from a TWIR issue.
///
/// # Returns
/// A list of [`Section`]s preserving the order found in the input.
pub fn parse_sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current: Option<Section> = None;
    let mut buffer = String::new();
    let parser = Parser::new_ext(text, Options::ENABLE_TABLES);
    let mut link_dest: Option<String> = None;
    let mut list_depth: usize = 0;
    let mut in_code_block = false;
    let mut in_heading = false;
    let mut heading_raw = String::new();
    let mut heading_sanitized = String::new();
    let mut table: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_raw.clear();
                heading_sanitized.clear();
                if level == HeadingLevel::H2 {
                    if let Some(ref mut sec) = current {
                        let line = buffer.trim();
                        if !line.is_empty() {
                            sec.lines.push(line.to_string());
                        }
                    }
                    if let Some(sec) = current.take() {
                        sections.push(sec);
                    }
                    buffer.clear();
                } else if matches!(
                    level,
                    HeadingLevel::H1 | HeadingLevel::H3 | HeadingLevel::H4
                ) {
                    if let Some(ref mut sec) = current {
                        let line = buffer.trim_end();
                        if !line.is_empty() {
                            sec.lines.push(line.to_string());
                        }
                    }
                    buffer.clear();
                }
            }
            Event::End(TagEnd::Heading(level)) => {
                in_heading = false;
                let raw = heading_raw.trim();
                if level == HeadingLevel::H2 {
                    current = Some(Section {
                        title: raw.to_string(),
                        lines: Vec::new(),
                    });
                    buffer.clear();
                } else if matches!(
                    level,
                    HeadingLevel::H1 | HeadingLevel::H3 | HeadingLevel::H4
                ) {
                    if let Some(ref mut sec) = current
                        && !raw.is_empty()
                    {
                        sec.lines.push(format_subheading(raw));
                    }
                    buffer.clear();
                } else if let Some(ref mut sec) = current {
                    let sanitized = heading_sanitized.trim();
                    if !sanitized.is_empty() {
                        let fixed = replace_github_mentions(&fix_bare_link(sanitized));
                        sec.lines.push(fixed);
                    }
                    buffer.clear();
                } else {
                    buffer.clear();
                }
                heading_raw.clear();
                heading_sanitized.clear();
            }
            Event::Start(Tag::List(_)) => {
                if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = replace_github_mentions(&fix_bare_link(line));
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        sec.lines.push(format!("{indent}• {fixed}"));
                        buffer.clear();
                    }
                }
                list_depth += 1;
            }
            Event::End(TagEnd::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
            }
            Event::Start(Tag::Item) => {
                buffer.clear();
            }
            Event::End(TagEnd::Item) => {
                if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = replace_github_mentions(&fix_bare_link(line));
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        sec.lines.push(format!("{indent}• {fixed}"));
                    }
                }
                buffer.clear();
            }
            Event::Start(Tag::Table(_)) => {
                table.clear();
                if !buffer.trim().is_empty() {
                    if let Some(ref mut sec) = current {
                        sec.lines.push(buffer.trim().to_string());
                    }
                    buffer.clear();
                }
            }
            Event::Start(Tag::TableHead) => {
                row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                table.push(row.clone());
            }
            Event::End(TagEnd::Table) => {
                if let Some(ref mut sec) = current {
                    for r in table.drain(..) {
                        let mut line = String::new();
                        for (i, cell) in r.into_iter().enumerate() {
                            if i == 0 {
                                line.push_str("\\| ");
                            } else {
                                line.push_str(" \\| ");
                            }
                            line.push_str(&cell);
                        }
                        line.push_str(" \\|");
                        sec.lines.push(line);
                    }
                }
            }
            Event::Start(Tag::TableRow) => {
                row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                table.push(row.clone());
            }
            Event::Start(Tag::TableCell) => {
                buffer.clear();
            }
            Event::End(TagEnd::TableCell) => {
                row.push(normalize_table_text(buffer.trim()).to_string());
                buffer.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                if list_depth > 0 {
                    let trimmed = buffer.trim_end().to_string();
                    buffer.clear();
                    if !trimmed.is_empty() {
                        buffer.push_str(&trimmed);
                        buffer.push('\n');
                    }
                } else if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = replace_github_mentions(&fix_bare_link(line));
                        sec.lines.push(fixed);
                    }
                    buffer.clear();
                } else {
                    buffer.clear();
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                if in_heading {
                    heading_raw.push('[');
                    heading_sanitized.push('[');
                } else {
                    buffer.push('[');
                }
                link_dest = Some(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => {
                if let Some(d) = link_dest.take() {
                    if in_heading {
                        heading_raw.push_str("](");
                        heading_raw.push_str(&d);
                        heading_raw.push(')');
                        heading_sanitized.push_str("](");
                        heading_sanitized.push_str(&escape_markdown_url(&d));
                        heading_sanitized.push(')');
                    } else {
                        buffer.push_str("](");
                        buffer.push_str(&escape_markdown_url(&d));
                        buffer.push(')');
                    }
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                if !buffer.is_empty() && !buffer.ends_with('\n') {
                    buffer.push('\n');
                }
                buffer.push_str("\\> ");
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                buffer.push('\n');
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                buffer.push_str("```\n");
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                if !buffer.ends_with('\n') {
                    buffer.push('\n');
                }
                buffer.push_str("```");
            }
            Event::Text(t) | Event::Code(t) => {
                if in_heading {
                    heading_raw.push_str(&t);
                    heading_sanitized.push_str(&escape(&t));
                } else if in_code_block {
                    buffer.push_str(&t);
                } else {
                    buffer.push_str(&escape(&t));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_code_block {
                    buffer.push('\n');
                } else if in_heading {
                    heading_raw.push(' ');
                    heading_sanitized.push(' ');
                } else {
                    buffer.push(' ');
                }
            }
            Event::Html(html) => {
                if html.trim_start().starts_with("<br") {
                    if in_heading {
                        heading_raw.push(' ');
                        heading_sanitized.push(' ');
                    } else {
                        buffer.push(' ');
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(mut sec) = current {
        if !buffer.trim().is_empty() {
            let fixed = replace_github_mentions(&fix_bare_link(buffer.trim()));
            sec.lines.push(fixed);
        }
        sections.push(sec);
    }
    sections
}
