use std::fs;
use std::process::Command;

#[cfg(feature = "integration")]
use mockito::Matcher;
#[allow(dead_code)]
#[path = "../src/validator.rs"]
mod validator;

use validator::validate_telegram_markdown;

#[test]
fn crate_of_the_week_is_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: This Week in Rust 605\nNumber: 605\nDate: 2025-06-25\n\n## Crate of the Week\nThis week's crate is [primitive_fixed_point_decimal](https://docs.rs/primitive_fixed_point_decimal), a crate of *real* fixed-point decimal types.\n\nThanks to [Wu Bingzheng](https://users.rust-lang.org/t/crate-of-the-week/2704/1445) for the self-suggestion!\n\n[Please submit your suggestions and votes for next week][submit_crate]!\n\n[submit_crate]: https://users.rust-lang.org/t/crate-of-the-week/2704\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .status()
        .expect("failed to run binary");
    assert!(status.success());

    let output = fs::read_to_string(dir.path().join("output_1.md")).unwrap();
    assert!(output.contains("📰 **CRATE OF THE WEEK**"));
    assert!(output.contains("primitive\\_fixed\\_point\\_decimal"));
    validate_telegram_markdown(&output).unwrap();
}

#[test]
fn crate_of_week_followed_by_section() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: Test\nNumber: 1\nDate: 2024-01-01\n\n## Crate of the Week\nThis week's crate is [demo](https://example.com).\n\n## Next\n- item\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .status()
        .expect("failed to run binary");
    assert!(status.success());

    let first = fs::read_to_string(dir.path().join("output_1.md")).unwrap();
    let second = fs::read_to_string(dir.path().join("output_2.md")).unwrap();
    assert!(first.contains("📰 **CRATE OF THE WEEK**"));
    assert!(first.contains("[demo](https://example.com)"));
    assert!(second.contains("📰 **NEXT**"));
    validate_telegram_markdown(&first).unwrap();
    validate_telegram_markdown(&second).unwrap();
}

#[cfg(feature = "integration")]
#[test]
fn telegram_request_sent() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- item\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let mut server = mockito::Server::new();
    let m = server
        .mock("POST", "/botTEST/sendMessage")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded("chat_id".into(), "42".into()),
            Matcher::UrlEncoded("parse_mode".into(), "MarkdownV2".into()),
            Matcher::UrlEncoded("disable_web_page_preview".into(), "true".into()),
        ]))
        .with_status(200)
        .with_body("{\"ok\":true}")
        .expect(2)
        .create();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .env("TELEGRAM_BOT_TOKEN", "TEST")
        .env("TELEGRAM_CHAT_ID", "42")
        .env("TELEGRAM_API_BASE", server.url())
        .status()
        .expect("failed to run binary");
    assert!(status.success());
    let post1 = fs::read_to_string(dir.path().join("output_1.md")).unwrap();
    let post2 = fs::read_to_string(dir.path().join("output_2.md")).unwrap();
    validate_telegram_markdown(&post1).unwrap();
    validate_telegram_markdown(&post2).unwrap();
    m.assert();
}

#[test]
fn fails_on_unescaped_markdown() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- bad *text\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .env("TELEGRAM_BOT_TOKEN", "TEST")
        .env("TELEGRAM_CHAT_ID", "42")
        // Use example.invalid to avoid accidental network calls
        .env("TELEGRAM_API_BASE", "http://example.invalid")
        .status()
        .expect("failed to run binary");
    assert!(!status.success());
}

#[test]
fn fails_on_unescaped_dash() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\nsome - text\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .env("TELEGRAM_BOT_TOKEN", "TEST")
        .env("TELEGRAM_CHAT_ID", "42")
        // Use example.invalid to avoid accidental network calls
        .env("TELEGRAM_API_BASE", "http://example.invalid")
        .status()
        .expect("failed to run binary");
    assert!(!status.success());
}

#[cfg(feature = "integration")]
#[test]
fn telegram_request_sent_plain() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- item\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let mut server = mockito::Server::new();
    let m = server
        .mock("POST", "/botTEST/sendMessage")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_request(|req| {
            let body = req.utf8_lossy_body().unwrap();
            body.contains("chat_id=42")
                && body.contains("disable_web_page_preview=true")
                && !body.contains("parse_mode")
        })
        .with_status(200)
        .with_body("{\"ok\":true}")
        .expect(2)
        .create();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .arg("--plain")
        .current_dir(dir.path())
        .env("TELEGRAM_BOT_TOKEN", "TEST")
        .env("TELEGRAM_CHAT_ID", "42")
        .env("TELEGRAM_API_BASE", server.url())
        .status()
        .expect("failed to run binary");
    assert!(status.success());
    let _post1 = fs::read_to_string(dir.path().join("output_1.md")).unwrap();
    let _post2 = fs::read_to_string(dir.path().join("output_2.md")).unwrap();
    m.assert();
}

#[cfg(feature = "integration")]
#[test]
fn sends_valid_markdown() {
    let dir = tempfile::tempdir().unwrap();
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- **Bold**\n";
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let mut server = mockito::Server::new();
    let m = server
        .mock("POST", "/botTEST/sendMessage")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded("chat_id".into(), "42".into()),
            Matcher::UrlEncoded("parse_mode".into(), "MarkdownV2".into()),
            Matcher::UrlEncoded("disable_web_page_preview".into(), "true".into()),
        ]))
        .with_status(200)
        .with_body("{\"ok\":true}")
        .expect(2)
        .create();

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .env("TELEGRAM_BOT_TOKEN", "TEST")
        .env("TELEGRAM_CHAT_ID", "42")
        .env("TELEGRAM_API_BASE", server.url())
        .status()
        .expect("failed to run binary");
    assert!(status.success());
    let post1 = fs::read_to_string(dir.path().join("output_1.md")).unwrap();
    let post2 = fs::read_to_string(dir.path().join("output_2.md")).unwrap();
    validate_telegram_markdown(&post1).unwrap();
    validate_telegram_markdown(&post2).unwrap();
    m.assert();
}

#[cfg(feature = "integration")]
#[test]
fn full_issue_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let input = include_str!("2025-07-02-this-week-in-rust.md");
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let mut server = mockito::Server::new();
    let mut mocks = Vec::new();
    // Issue 606 currently generates 11 posts, so expect that
    // many requests to the mock server.
    for _ in 0..11 {
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

    let status = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"))
        .arg(&input_path)
        .current_dir(dir.path())
        .env("TELEGRAM_BOT_TOKEN", "TEST")
        .env("TELEGRAM_CHAT_ID", "42")
        .env("TELEGRAM_API_BASE", server.url())
        .status()
        .expect("failed to run binary");
    assert!(status.success());

    for i in 1..=8 {
        let post = fs::read_to_string(dir.path().join(format!("output_{i}.md"))).unwrap();
        validate_telegram_markdown(&post).unwrap();
    }
    for m in mocks {
        m.assert();
    }
}
