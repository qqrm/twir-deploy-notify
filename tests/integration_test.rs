#[allow(dead_code)]
#[path = "../src/generator.rs"]
mod generator;
#[allow(dead_code)]
#[path = "../src/parser.rs"]
mod parser;
#[allow(dead_code)]
#[path = "../src/validator.rs"]
mod validator;

use generator::generate_posts;
use validator::validate_telegram_markdown;

#[test]
fn parse_latest_issue_full() {
    let input = include_str!("2025-06-25-this-week-in-rust.md");
    let posts = generate_posts(input.to_string());

    let expected = [
        include_str!("expected/expected1.md"),
        include_str!("expected/expected2.md"),
        include_str!("expected/expected3.md"),
        include_str!("expected/expected4.md"),
        include_str!("expected/expected5.md"),
        include_str!("expected/expected6.md"),
        include_str!("expected/expected7.md"),
        include_str!("expected/expected8.md"),
        include_str!("expected/expected9.md"),
        include_str!("expected/expected10.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn parse_complex_markdown() {
    let input = include_str!("complex.md");
    let posts = generate_posts(input.to_string());

    let expected = [include_str!("expected/complex1.md")];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn parse_issue_606_full() {
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let posts = generate_posts(input.to_string());

    let expected = [
        include_str!("expected/606_1.md"),
        include_str!("expected/606_2.md"),
        include_str!("expected/606_3.md"),
        include_str!("expected/606_4.md"),
        include_str!("expected/606_5.md"),
        include_str!("expected/606_6.md"),
        include_str!("expected/606_7.md"),
        include_str!("expected/606_8.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn validate_generated_posts() {
    let input = include_str!("2025-06-25-this-week-in-rust.md");
    let posts = generate_posts(input.to_string());
    assert!(!posts.is_empty());
    for (i, post) in posts.iter().enumerate() {
        validate_telegram_markdown(post)
            .unwrap_or_else(|e| panic!("post {} invalid: {}", i + 1, e));
    }
}
