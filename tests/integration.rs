#[cfg(feature = "integration")]
use std::fs;
#[cfg(feature = "integration")]
use std::process::Command;
use twir_deploy_notify::generator;

#[cfg(feature = "integration")]
use mockito::Matcher;

mod common;

#[cfg(feature = "integration")]
#[test]
fn full_issue_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let mut server = mockito::Server::new();
    let mut mocks = Vec::new();
    // Issue 606 currently generates 5 posts, so expect that
    // many requests to the mock server.
    for _ in 0..5 {
        mocks.push(
            server
                .mock("POST", "/botTEST/sendMessage")
                .match_header("content-type", "application/x-www-form-urlencoded")
                .match_body(Matcher::AllOf(vec![
                    Matcher::UrlEncoded("chat_id".into(), "-10042".into()),
                    Matcher::UrlEncoded("parse_mode".into(), "MarkdownV2".into()),
                    Matcher::UrlEncoded("disable_web_page_preview".into(), "true".into()),
                ]))
                .with_status(200)
                .with_body("{\"ok\":true,\"result\":{\"message_id\":1}}")
                .create(),
        );
    }

    let m_pin = server
        .mock("POST", "/botTEST/pinChatMessage")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded("chat_id".into(), "-10042".into()),
            Matcher::UrlEncoded("message_id".into(), "1".into()),
        ]))
        .with_status(200)
        .with_body("{\"ok\":true,\"result\":true}")
        .expect(1)
        .create();

    let m_del = server
        .mock("POST", "/botTEST/deleteMessage")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded("chat_id".into(), "-10042".into()),
            Matcher::UrlEncoded("message_id".into(), "2".into()),
        ]))
        .with_status(200)
        .with_body("{\"ok\":true,\"result\":true}")
        .expect(1)
        .create();

    mocks.push(m_pin);
    mocks.push(m_del);

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .env("TELEGRAM_BOT_TOKEN", "TEST")
        .env("TELEGRAM_CHAT_ID", "-10042")
        .env("TELEGRAM_API_BASE", server.url())
        .status()
        .expect("failed to run binary");
    assert!(status.success());

    for i in 1..=5 {
        let post = fs::read_to_string(dir.path().join(format!("output_{i}.md"))).unwrap();
        common::assert_valid_markdown(&post);
    }
    for m in mocks {
        m.assert();
    }
}

#[test]
fn parse_latest_issue_full() {
    let input = include_str!("2025-06-25-this-week-in-rust.md");
    let posts = generator::generate_posts(input.to_string()).unwrap();

    let expected = [
        include_str!("expected/expected1.md"),
        include_str!("expected/expected2.md"),
        include_str!("expected/expected3.md"),
        include_str!("expected/expected4.md"),
        include_str!("expected/expected5.md"),
        include_str!("expected/expected6.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn parse_complex_markdown() {
    let input = include_str!("complex.md");
    let posts = generator::generate_posts(input.to_string()).unwrap();

    let expected = [include_str!("expected/complex1.md")];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn parse_issue_606_full() {
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let posts = generator::generate_posts(input.to_string()).unwrap();

    let expected = [
        include_str!("expected/606_1.md"),
        include_str!("expected/606_2.md"),
        include_str!("expected/606_3.md"),
        include_str!("expected/606_4.md"),
        include_str!("expected/606_5.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
    }
}

#[test]
fn parse_issue_607_full() {
    let input = include_str!("2025-07-05-this-week-in-rust.md");
    let posts = generator::generate_posts(input.to_string()).unwrap();

    let expected = [
        include_str!("expected/607_1.md"),
        include_str!("expected/607_2.md"),
        include_str!("expected/607_3.md"),
        include_str!("expected/607_4.md"),
        include_str!("expected/607_5.md"),
    ];

    assert_eq!(posts.len(), expected.len(), "post count mismatch");
    for (i, (post, exp)) in posts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(post, exp, "Mismatch in post {}", i + 1);
        common::assert_valid_markdown(post);
    }
}

#[test]
fn validate_fixture_posts() {
    let fixtures = [
        include_str!("2025-06-25-this-week-in-rust.md"),
        include_str!("2025-07-02-this-week-in-rust.md"),
        include_str!("complex.md"),
    ];

    for (idx, input) in fixtures.iter().enumerate() {
        let posts = generator::generate_posts((*input).to_string()).unwrap();
        assert!(
            !posts.is_empty(),
            "no posts generated for fixture {}",
            idx + 1
        );
        for post in &posts {
            common::assert_valid_markdown(post);
        }
    }
}

#[test]
fn issue_606_no_unescaped_dashes() {
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let posts = generator::generate_posts(input.to_string()).unwrap();
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
fn jobs_links_present() {
    let input = include_str!("2025-07-05-this-week-in-rust.md");
    let posts = generator::generate_posts(input.to_string()).unwrap();
    let combined = posts.join("\n");
    assert!(combined.contains("[Rust Job Reddit Thread](https://www.reddit.com/r/rust/comments/1llcso7/official_rrust_whos_hiring_thread_for_jobseekers/)"));
    assert!(combined.contains("[Rust Jobs chat](https://t.me/rust_jobs)"));
    assert!(combined.contains("[Rust Jobs feed](https://t.me/rust_jobs_feed)"));
}
