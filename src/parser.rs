use once_cell::sync::Lazy;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};
use regex::Regex;

use crate::generator::{escape_markdown, escape_markdown_url, format_subheading};

/// Representation of a single TWIR section.
#[derive(Default)]
pub struct Section {
    pub title: String,
    pub lines: Vec<String>,
}

static BARE_LINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\((https?://.*)\)\s*$").unwrap());

fn fix_bare_link(line: &str) -> String {
    if line.contains("](") {
        return line.to_string();
    }
    let plain = line.replace('\\', "");
    if let Some(caps) = BARE_LINK_RE.captures(&plain) {
        let url_raw = caps.get(1).unwrap().as_str();
        let url = url_raw.replace('\\', "");
        let text = plain[..caps.get(0).unwrap().start()].trim_end();
        format!("[{}]({})", escape_markdown(text), escape_markdown_url(&url))
    } else {
        line.to_string()
    }
}

/// Parse TWIR Markdown into sections using `pulldown-cmark`.
pub fn parse_sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current: Option<Section> = None;
    let mut buffer = String::new();
    let parser = Parser::new_ext(text, Options::ENABLE_TABLES);
    let mut link_dest: Option<String> = None;
    let mut list_depth: usize = 0;
    let mut in_code_block = false;
    let mut table: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    for event in parser {
        match event {
            Event::Start(Tag::Heading(HeadingLevel::H2, ..)) => {
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
            }
            Event::End(Tag::Heading(HeadingLevel::H2, ..)) => {
                current = Some(Section {
                    title: buffer.trim().to_string(),
                    lines: Vec::new(),
                });
                buffer.clear();
            }
            Event::Start(Tag::Heading(HeadingLevel::H3 | HeadingLevel::H4, ..)) => {
                if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        sec.lines.push(line.to_string());
                    }
                }
                buffer.clear();
            }
            Event::End(Tag::Heading(HeadingLevel::H3 | HeadingLevel::H4, ..)) => {
                if let Some(ref mut sec) = current {
                    let heading = buffer.trim();
                    if !heading.is_empty() {
                        sec.lines.push(format_subheading(heading));
                    }
                }
                buffer.clear();
            }
            Event::Start(Tag::List(_)) => {
                if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = fix_bare_link(line);
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        sec.lines.push(format!("{indent}• {fixed}"));
                        buffer.clear();
                    }
                }
                list_depth += 1;
            }
            Event::End(Tag::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
            }
            Event::Start(Tag::Item) => {
                buffer.clear();
            }
            Event::End(Tag::Item) => {
                if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = fix_bare_link(line);
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
            Event::End(Tag::TableHead) => {
                table.push(row.clone());
            }
            Event::End(Tag::Table(_)) => {
                if let Some(ref mut sec) = current {
                    let mut widths: Vec<usize> = vec![];
                    for r in &table {
                        for (i, cell) in r.iter().enumerate() {
                            if i >= widths.len() {
                                widths.push(cell.len());
                            } else if widths[i] < cell.len() {
                                widths[i] = cell.len();
                            }
                        }
                    }
                    for r in table.drain(..) {
                        let mut line = String::from("|");
                        for (i, cell) in r.into_iter().enumerate() {
                            let width = widths[i];
                            line.push_str(&format!(" {cell:width$} |"));
                        }
                        sec.lines.push(line);
                    }
                }
            }
            Event::Start(Tag::TableRow) => {
                row.clear();
            }
            Event::End(Tag::TableRow) => {
                table.push(row.clone());
            }
            Event::Start(Tag::TableCell) => {
                buffer.clear();
            }
            Event::End(Tag::TableCell) => {
                row.push(buffer.trim().to_string());
                buffer.clear();
            }
            Event::End(Tag::Paragraph) => {
                if let Some(ref mut sec) = current {
                    let line = buffer.trim_end();
                    if !line.is_empty() {
                        let fixed = fix_bare_link(line);
                        sec.lines.push(fixed);
                    }
                }
                buffer.clear();
            }
            Event::Start(Tag::Link(_, dest, _)) => {
                buffer.push('[');
                link_dest = Some(dest.to_string());
            }
            Event::End(Tag::Link(_, _, _)) => {
                if let Some(d) = link_dest.take() {
                    buffer.push_str("](");
                    buffer.push_str(&escape_markdown_url(&d));
                    buffer.push(')');
                }
            }
            Event::Start(Tag::BlockQuote) => {
                buffer.push_str("\\> ");
            }
            Event::End(Tag::BlockQuote) => {
                buffer.push('\n');
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                buffer.push_str("```\n");
            }
            Event::End(Tag::CodeBlock(_)) => {
                in_code_block = false;
                if !buffer.ends_with('\n') {
                    buffer.push('\n');
                }
                buffer.push_str("```");
            }
            Event::Text(t) | Event::Code(t) => {
                buffer.push_str(&escape_markdown(&t));
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_code_block {
                    buffer.push('\n');
                } else {
                    buffer.push(' ');
                }
            }
            _ => {}
        }
    }
    if let Some(mut sec) = current {
        if !buffer.trim().is_empty() {
            sec.lines.push(buffer.trim().to_string());
        }
        sections.push(sec);
    }
    sections
}
