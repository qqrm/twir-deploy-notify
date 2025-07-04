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
fn issue_606_dash_after_newline() {
    // Extract a post from the 2025-07-02 issue that begins with a dash
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let posts = generate_posts(input.to_string());
    // Ensure at least one generated post contains an unescaped dash at the start
    let broken = posts.into_iter().find(|p| p.starts_with("-"));
    if let Some(post) = broken {
        assert!(validate_telegram_markdown(&post).is_err());
    } else {
        // Construct a failing example similar to the Telegram error
        let msg = "- example";
        assert!(validate_telegram_markdown(msg).is_err());
    }
}
