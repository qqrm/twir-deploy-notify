use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};
use regex::Regex;
use std::{env, fs, path::Path};
use teloxide::utils::markdown::{escape, escape_link_url};

/// Representation of a single TWIR section.
#[derive(Default)]
struct Section {
    title: String,
    lines: Vec<String>,
}

pub const TELEGRAM_LIMIT: usize = 4000;

/// Escape characters for MarkdownV2
pub fn escape_markdown(text: &str) -> String {
    escape(text)
}

/// Escape characters for the URL part in [text](url)
pub fn escape_markdown_url(url: &str) -> String {
    escape_link_url(url)
}

/// Escape characters for MarkdownV2 URLs using `teloxide` utilities.
pub fn escape_url(url: &str) -> String {
    escape_markdown_url(url)
}

/// Format a section heading with an emoji and uppercase text
pub fn format_heading(title: &str) -> String {
    let upper = title.to_uppercase();
    format!("📰 **{}**", escape_markdown(&upper))
}

/// Format a subheading without emoji
pub fn format_subheading(title: &str) -> String {
    format!("**{}**", escape_markdown(title))
}

/// Convert Markdown-formatted text into plain text with URLs in parentheses
pub fn markdown_to_plain(text: &str) -> String {
    let without_escapes = text.replace('\\', "");
    let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    let replaced = link_re.replace_all(&without_escapes, "$1 ($2)");
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

/// Split long text into multiple messages
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

/// Convert "text (url)" into "[text](url)" if no link formatting is present.
fn fix_bare_link(line: &str) -> String {
    if line.contains("](") {
        return line.to_string();
    }
    let re = Regex::new(r"\\((https?://[^\\)]+)\\)\s*$").unwrap();
    if let Some(caps) = re.captures(line) {
        let url = caps.get(1).unwrap().as_str();
        let text = line[..caps.get(0).unwrap().start()].trim_end();
        format!("[{text}]({url})")
    } else {
        line.to_string()
    }
}

/// Parse TWIR Markdown into sections using `pulldown-cmark`.
fn parse_sections(text: &str) -> Vec<Section> {
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
                    // compute column widths
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

/// Main function generating Telegram posts from Markdown
pub fn generate_posts(mut input: String) -> Vec<String> {
    let title_re = Regex::new(r"(?m)^Title: (.+)$").unwrap();
    let number_re = Regex::new(r"(?m)^Number: (.+)$").unwrap();
    let date_re = Regex::new(r"(?m)^Date: (.+)$").unwrap();

    let title = title_re
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    let number = number_re
        .captures(&input)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());
    let date = date_re
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

    let header_re = Regex::new(r"(?m)^(Title|Number|Date):.*$\n?").unwrap();
    let body = header_re.replace_all(&input, "");
    let sections = parse_sections(&body);

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
        section_text.push_str(&format!("{}\n", format_heading(&sec.title)));
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

    if let Some(link) = url {
        let link_block = format!(
            "\n\\-\\-\\-\n\nПолный выпуск: [{}]({})",
            escape_markdown(&link),
            escape_url(&link)
        );
        if current.len() + link_block.len() > TELEGRAM_LIMIT && !current.is_empty() {
            posts.push(current.clone());
            current = link_block;
        } else {
            current.push_str(&link_block);
        }
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

/// Write Telegram posts to files
pub fn write_posts(posts: &[String], dir: &Path) -> std::io::Result<()> {
    for (i, post) in posts.iter().enumerate() {
        let file_name = dir.join(format!("output_{}.md", i + 1));
        fs::write(&file_name, post)?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut input_path = "input.md";
    let mut plain = false;

    for arg in &args[1..] {
        if arg == "--plain" {
            plain = true;
        } else {
            input_path = arg;
        }
    }

    let input = fs::read_to_string(input_path)?;
    let mut posts = generate_posts(input);

    if plain {
        posts = posts.into_iter().map(|p| markdown_to_plain(&p)).collect();
    }

    write_posts(&posts, Path::new("."))?;
    for (i, _) in posts.iter().enumerate() {
        println!("Generated output_{}.md", i + 1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let escaped = escape_url(url);
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

    mod property {
        use super::*;
        use proptest::prelude::*;

        fn arb_heading() -> impl Strategy<Value = String> {
            "[A-Za-z0-9 ]{1,20}".prop_map(|s| format!("## {}", s))
        }

        fn arb_list() -> impl Strategy<Value = String> {
            prop::collection::vec("[A-Za-z0-9 ]{1,20}", 1..5).prop_map(|items| {
                items
                    .into_iter()
                    .map(|s| format!("- {}", s))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }

        fn arb_table() -> impl Strategy<Value = String> {
            prop::collection::vec(("[A-Za-z0-9 ]{1,10}", "[A-Za-z0-9 ]{1,10}"), 1..4).prop_map(
                |rows| {
                    let mut table = String::from("| Col1 | Col2 |\n|------|------|\n");
                    for (c1, c2) in rows {
                        table.push_str(&format!("| {} | {} |\n", c1, c2));
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
                .prop_map(|body| format!("Title: Test\nNumber: 1\nDate: 2025-01-01\n\n{}", body))
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
