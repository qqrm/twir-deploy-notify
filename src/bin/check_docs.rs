use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};
use std::{fs, path::Path};
use walkdir::WalkDir;

fn parse_rs_file(code: &str) -> Option<&str> {
    let trimmed = code.trim();
    trimmed
        .strip_suffix(".rs")
        .filter(|_| {
            !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'))
        })
        .map(|_| trimmed)
}

fn parse_function(code: &str) -> Option<&str> {
    let trimmed = code.trim();
    trimmed.strip_suffix('(').filter(|name| {
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

fn function_exists(name: &str) -> std::io::Result<bool> {
    for entry in WalkDir::new("src").into_iter().chain(WalkDir::new("tests")) {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) == Some("rs") {
            let content = fs::read_to_string(entry.path())?;
            if content.contains(&format!("fn {name}")) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut missing = Vec::new();

    for entry in WalkDir::new("DOCS").into_iter().filter_map(Result::ok) {
        if entry.path().extension().and_then(|e| e.to_str()) == Some("md") {
            let content = fs::read_to_string(entry.path())?;
            let parser = Parser::new(&content);
            let mut in_block = false;
            for event in parser {
                match event {
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => in_block = true,
                    Event::End(Tag::CodeBlock(_)) => in_block = false,
                    Event::Code(code) | Event::Text(code) if in_block => {
                        if let Some(p) = parse_rs_file(&code) {
                            if !Path::new(p).exists() {
                                missing
                                    .push(format!("{}: missing file {p}", entry.path().display()));
                            }
                        } else if let Some(name) = parse_function(&code) {
                            if !function_exists(name)? {
                                missing.push(format!(
                                    "{}: missing function {name}",
                                    entry.path().display()
                                ));
                            }
                        }
                    }
                    Event::Code(code) => {
                        if let Some(p) = parse_rs_file(&code) {
                            if !Path::new(p).exists() {
                                missing
                                    .push(format!("{}: missing file {p}", entry.path().display()));
                            }
                        } else if let Some(name) = parse_function(&code) {
                            if !function_exists(name)? {
                                missing.push(format!(
                                    "{}: missing function {name}",
                                    entry.path().display()
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if !missing.is_empty() {
        for m in &missing {
            eprintln!("{m}");
        }
        std::process::exit(1);
    }

    Ok(())
}
