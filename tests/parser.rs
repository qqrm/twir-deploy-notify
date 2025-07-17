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
fn regression_table_ascii_aligned() {
    let input = include_str!("regression_table.md");
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    let lines = &sections[0].lines;
    assert!(lines.first().is_some_and(|l| l == "```"));
    assert!(lines.last().is_some_and(|l| l == "```"));
    let body = &lines[1..lines.len() - 1];
    let width = body.first().unwrap().len();
    let positions: Vec<_> = body
        .first()
        .unwrap()
        .match_indices('|')
        .map(|(i, _)| i)
        .collect();
    for line in body {
        assert_eq!(line.len(), width);
        assert!(line.is_ascii());
        let pos: Vec<_> = line.match_indices('|').map(|(i, _)| i).collect();
        assert_eq!(pos, positions);
    }
}
