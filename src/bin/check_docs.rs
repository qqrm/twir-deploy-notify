use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};
use regex::Regex;
use std::{fs, path::Path};
use walkdir::WalkDir;

fn function_exists(name: &str) -> std::io::Result<bool> {
    for entry in WalkDir::new("src").into_iter().chain(WalkDir::new("tests")) {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) == Some("rs") {
            let content = fs::read_to_string(entry.path())?;
            if content.contains(&format!("fn {}", name)) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_re = Regex::new(r"^([A-Za-z0-9_./-]+\\.rs)$")?;
    let func_re = Regex::new(r"^([A-Za-z0-9_]+)\\()$")?;

    let mut missing = Vec::new();

    for entry in WalkDir::new(".").into_iter().filter_map(Result::ok) {
        if entry.path().extension().and_then(|e| e.to_str()) == Some("md") {
            let content = fs::read_to_string(entry.path())?;
            let parser = Parser::new(&content);
            let mut in_block = false;
            for event in parser {
                match event {
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => in_block = true,
                    Event::End(Tag::CodeBlock(_)) => in_block = false,
                    Event::Code(code) | Event::Text(code) if in_block => {
                        if let Some(cap) = file_re.captures(&code) {
                            let p = cap.get(1).unwrap().as_str();
                            if !Path::new(p).exists() {
                                missing
                                    .push(format!("{}: missing file {p}", entry.path().display()));
                            }
                        } else if let Some(cap) = func_re.captures(&code) {
                            let name = cap.get(1).unwrap().as_str();
                            if !function_exists(name)? {
                                missing.push(format!(
                                    "{}: missing function {name}",
                                    entry.path().display()
                                ));
                            }
                        }
                    }
                    Event::Code(code) => {
                        if let Some(cap) = file_re.captures(&code) {
                            let p = cap.get(1).unwrap().as_str();
                            if !Path::new(p).exists() {
                                missing
                                    .push(format!("{}: missing file {p}", entry.path().display()));
                            }
                        } else if let Some(cap) = func_re.captures(&code) {
                            let name = cap.get(1).unwrap().as_str();
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
