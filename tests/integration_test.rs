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
use generator::send_to_telegram;
use validator::validate_telegram_markdown;

#[test]
fn parse_latest_issue_full() {
    let input = include_str!("2025-06-25-this-week-in-rust.md");
    let posts = generate_posts(input.to_string()).unwrap();

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
        include_str!("expected/expected11.md"),
        include_str!("expected/expected12.md"),
        include_str!("expected/expected13.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn parse_complex_markdown() {
    let input = include_str!("complex.md");
    let posts = generate_posts(input.to_string()).unwrap();

    let expected = [
        include_str!("expected/complex1.md"),
        include_str!("expected/complex2.md"),
        include_str!("expected/complex3.md"),
        include_str!("expected/complex4.md"),
        include_str!("expected/complex5.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn parse_issue_606_full() {
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let posts = generate_posts(input.to_string()).unwrap();

    let expected = [
        include_str!("expected/606_1.md"),
        include_str!("expected/606_2.md"),
        include_str!("expected/606_3.md"),
        include_str!("expected/606_4.md"),
        include_str!("expected/606_5.md"),
        include_str!("expected/606_6.md"),
        include_str!("expected/606_7.md"),
        include_str!("expected/606_8.md"),
        include_str!("expected/606_9.md"),
        include_str!("expected/606_10.md"),
        include_str!("expected/606_11.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn parse_call_for_participation() {
    let input = include_str!("2025-07-05-call-for-participation.md");
    let posts = generate_posts(input.to_string());

    let expected = [
        include_str!("expected/cfp1.md"),
        include_str!("expected/cfp2.md"),
  ];
}

#[test]    
fn parse_issue_607_full() {
    let input = include_str!("2025-07-05-this-week-in-rust.md");
    let posts = generate_posts(input.to_string());

    let expected = [
        include_str!("expected/607_1.md"),
        include_str!("expected/607_2.md"),
        include_str!("expected/607_3.md"),
        include_str!("expected/607_4.md"),
        include_str!("expected/607_5.md"),
        include_str!("expected/607_6.md"),
        include_str!("expected/607_7.md"),
        include_str!("expected/607_8.md"),
        include_str!("expected/607_9.md"),
        include_str!("expected/607_10.md"),
        include_str!("expected/607_11.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
        validate_telegram_markdown(post)
            .unwrap_or_else(|e| panic!("post {} invalid: {}", i + 1, e));
    }
}

#[test]
fn validate_generated_posts() {
    let input = include_str!("2025-06-25-this-week-in-rust.md");
    let posts = generate_posts(input.to_string()).unwrap();
    assert!(!posts.is_empty());
    for (i, post) in posts.iter().enumerate() {
        validate_telegram_markdown(post)
            .unwrap_or_else(|e| panic!("post {} invalid: {}", i + 1, e));
    }
}

#[test]
fn validate_issue_606_posts() {
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let posts = generate_posts(input.to_string()).unwrap();
    assert!(!posts.is_empty());
    for (i, post) in posts.iter().enumerate() {
        validate_telegram_markdown(post)
            .unwrap_or_else(|e| panic!("post {} invalid: {}", i + 1, e));
    }
}

#[test]
fn validate_issue_606_post_4() {
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let posts = generate_posts(input.to_string()).unwrap();
    assert!(posts.len() >= 4);
    validate_telegram_markdown(&posts[3]).unwrap();
}

#[test]
fn validate_complex_posts() {
    let input = include_str!("complex.md");
    let posts = generate_posts(input.to_string()).unwrap();
    assert!(!posts.is_empty());
    for (i, post) in posts.iter().enumerate() {
        validate_telegram_markdown(post)
            .unwrap_or_else(|e| panic!("post {} invalid: {}", i + 1, e));
    }
}

#[test]
fn send_long_escaped_dash() {
    use mockito::Matcher;

    let prefix = "a".repeat(generator::TELEGRAM_LIMIT - 1);
    let input = format!("Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n{prefix}\\-b");
    let posts = generate_posts(input).unwrap();
    assert!(!posts.is_empty());

    let mut server = mockito::Server::new();
    let mut mocks = Vec::new();
    for _ in 0..posts.len() {
        mocks.push(
            server
                .mock("POST", "/botTEST/sendMessage")
                .match_header("content-type", "application/x-www-form-urlencoded")
                .match_body(Matcher::AllOf(vec![
                    Matcher::UrlEncoded("chat_id".into(), "42".into()),
                    Matcher::UrlEncoded("parse_mode".into(), "MarkdownV2".into()),
                    Matcher::UrlEncoded("disable_web_page_preview".into(), "true".into()),
                ]))
                .with_status(200)
                .with_body("{\"ok\":true}")
                .create(),
        );
    }

    let result = send_to_telegram(&posts, &server.url(), "TEST", "42", true);
    assert!(result.is_ok(), "send_to_telegram failed: {result:?}");
    for m in mocks {
        m.assert();
    }
}

#[test]
fn issue_606_no_unescaped_dashes() {
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let posts = generate_posts(input.to_string());
    assert!(!posts.is_empty());
    for (i, post) in posts.iter().enumerate() {
        assert!(
            !post.starts_with('-'),
            "post {} begins with unescaped dash",
            i + 1
        );
    }
}

#[test]
fn parse_call_for_participation() {
    let input = include_str!("2025-07-05-call-for-participation.md");
    let posts = generate_posts(input.to_string());

    let expected = [
        include_str!("expected/cfp1.md"),
        include_str!("expected/cfp2.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
        validate_telegram_markdown(post)
            .unwrap_or_else(|e| panic!("post {} invalid: {}", i + 1, e));
    }
}
