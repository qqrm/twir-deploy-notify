mod common;
#[allow(dead_code)]
#[path = "../src/generator.rs"]
mod generator;
#[allow(dead_code)]
#[path = "../src/parser.rs"]
mod parser;
#[allow(dead_code)]
#[path = "../src/validator.rs"]
mod validator;

use common::assert_valid_markdown;
use parser::parse_sections;

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
        vec!["• [Some text](https://example.com/path(1\\))"]
    );
    assert_valid_markdown(&sections[0].lines[0]);
}
