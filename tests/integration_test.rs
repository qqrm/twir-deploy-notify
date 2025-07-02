#[allow(dead_code)]
#[path = "../src/main.rs"]
mod app;

use app::generate_posts;

#[test]
fn parse_latest_issue() {
    let input = include_str!("2025-06-25-this-week-in-rust.md");
    let posts = generate_posts(input.to_string());
    assert!(!posts.is_empty());
    // Ensure all posts contain text
    assert!(posts.iter().all(|p| !p.is_empty()));
    let combined = posts.join("\n");
    assert!(combined.contains("Полный выпуск"));
}
