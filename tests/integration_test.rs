#[allow(dead_code)]
#[path = "../src/main.rs"]
mod app;

use app::generate_posts;

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
