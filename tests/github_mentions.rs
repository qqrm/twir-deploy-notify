use twir_deploy_notify::parser;

mod common;
use parser::parse_sections;

#[test]
fn github_mentions_are_linked() {
    let input = "## Section\n- Thanks to @user for the fix";
    let sections = parse_sections(input);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].title, "Section");
    assert_eq!(
        sections[0].lines,
        vec!["• Thanks to [user](https://github.com/user) for the fix"]
    );
    common::assert_valid_markdown(&sections[0].lines[0]);
}
