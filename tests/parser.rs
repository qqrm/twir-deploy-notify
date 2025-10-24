use twir_deploy_notify::parser;

use parser::parse_sections;
mod common;

#[test]
fn code_block_before_next_heading() {
    let input = "## First\n```\nline1\nline2\n```\n\n## Second";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].title, "First");
    assert_eq!(sections[0].lines, vec!["```\nline1\nline2\n```"]);
}

#[test]
fn bare_link_with_parentheses() {
    let input = "## Section\n- Some text (https://example.com/path(1))";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].title, "Section");
    assert_eq!(
        sections[0].lines,
        vec!["• [Some text](https://example.com/path\\(1\\))"]
    );
    common::assert_valid_markdown(&sections[0].lines[0]);
}

#[test]
fn fenced_code_block_round_trip() {
    let input = "## Section\n```\nfn main() {\n    println!(\"Hello_world\");\n}\n```";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    assert_eq!(
        sections[0].lines,
        vec!["```\nfn main() {\n    println!(\"Hello_world\");\n}\n```"]
    );
}

#[test]
fn loose_list_preserves_parent_items() {
    let input = "## Section\n[Cargo](https://example.com/cargo)\n* [Parent](https://example.com/parent)\n  * [Child](https://example.com/child)\n\n* *No calls for testing were issued by [Rust](https://example.com/rust).*";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    let lines = &sections[0].lines;
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "[Cargo](https://example.com/cargo)");
    assert_eq!(lines[1], "• [Parent](https://example.com/parent)");
    assert_eq!(lines[2], "  • [Child](https://example.com/child)");
    assert_eq!(
        lines[3],
        "• No calls for testing were issued by [Rust](https://example.com/rust)\\."
    );
}

#[test]
fn regression_table_ascii_aligned() {
    let input = include_str!("regression_table.md");
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    let lines = &sections[0].lines;
    assert!(lines.first().is_none_or(|l| l != "```"));
    for line in lines {
        assert!(line.is_ascii());
        assert!(line.contains("\\|"));
    }
}

#[test]
fn regression_table_compact() {
    let input = include_str!("regression_table.md");
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    let lines = &sections[0].lines;
    assert!(lines.first().is_none_or(|l| l != "```"));
    assert!(lines.iter().any(|l| l.contains("\\|")));
}
