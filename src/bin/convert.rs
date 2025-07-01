use pulldown_cmark::{Event, Parser, Tag};
use std::fs;

const TELEGRAM_LIMIT: usize = 4000;

/// Escape special characters for Telegram MarkdownV2.
fn escape_telegram_md(text: &str) -> String {
    let specials = "_*[]()~`>#+-=|{}.!";
    let mut res = String::with_capacity(text.len());
    for c in text.chars() {
        if specials.contains(c) {
            res.push('\\');
        }
        res.push(c);
    }
    res
}

/// Convert common Markdown to Telegram MarkdownV2.
fn markdown_to_telegram(md: &str) -> String {
    let parser = Parser::new(md);
    let mut out = String::new();
    for event in parser {
        match event {
            Event::Start(Tag::Heading(level)) => {
                if level == 1 {
                    out.push_str("\n🦀 *");
                } else {
                    out.push_str("\n*");
                }
            }
            Event::End(Tag::Heading(_)) => out.push_str("*\n"),
            Event::Start(Tag::List(_)) => {}
            Event::End(Tag::List(_)) => out.push('\n'),
            Event::Start(Tag::Item) => out.push_str("• "),
            Event::End(Tag::Item) => out.push('\n'),
            Event::Start(Tag::Emphasis) => out.push('_'),
            Event::End(Tag::Emphasis) => out.push('_'),
            Event::Start(Tag::Strong) => out.push('*'),
            Event::End(Tag::Strong) => out.push('*'),
            Event::Text(t) => out.push_str(&escape_telegram_md(&t)),
            Event::Start(Tag::Link(_href, _url, _title)) => out.push('['),
            Event::End(Tag::Link(_href, url, _title)) => {
                out.push(']');
                out.push('(');
                out.push_str(&url);
                out.push(')');
            }
            Event::Code(t) => {
                out.push('`');
                out.push_str(&escape_telegram_md(&t));
                out.push('`');
            }
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            _ => {}
        }
    }
    out
}

/// Split long text into multiple messages without breaking lines.
fn split_posts(text: &str, limit: usize) -> Vec<String> {
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

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let input_path = args.get(1).map(String::as_str).unwrap_or("input.md");
    let md = fs::read_to_string(input_path)?;
    let converted = markdown_to_telegram(&md);
    let posts = split_posts(&converted, TELEGRAM_LIMIT);
    for (i, post) in posts.iter().enumerate() {
        let file_name = format!("tg_output_{}.md", i + 1);
        fs::write(&file_name, post)?;
        println!("Generated {}", file_name);
    }
    Ok(())
}
