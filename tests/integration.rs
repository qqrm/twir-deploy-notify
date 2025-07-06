use std::fs;
use std::process::Command;
#[cfg(feature = "integration")]
use twir_deploy_notify::generator;

#[cfg(feature = "integration")]
use mockito::Matcher;

mod common;

#[cfg(feature = "integration")]
fn run_single_post(input: &str, plain: bool, validate_markdown: bool) {
    use mockito::Matcher;

    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("input.md");
    fs::write(&input_path, input).unwrap();

    let mut server = mockito::Server::new();
    let mock = if plain {
        server
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
            .expect(1)
            .create()
    } else {
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
            .expect(1)
            .create()
    };

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_twir-deploy-notify"));
    cmd.arg(&input_path)
        .current_dir(dir.path())
        .env("TELEGRAM_BOT_TOKEN", "TEST")
        .env("TELEGRAM_CHAT_ID", "42")
        .env("TELEGRAM_API_BASE", server.url());
    if plain {
        cmd.arg("--plain");
    }

    let status = cmd.status().expect("failed to run binary");
    assert!(status.success());

    let post = fs::read_to_string(dir.path().join("output_1.md")).unwrap();
    if validate_markdown {
        common::assert_valid_markdown(&post);
    }
    mock.assert();
}

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
    assert!(output.contains("📦 **CRATE OF THE WEEK** 📦"));
    assert!(output.contains("primitive\\_fixed\\_point\\_decimal"));
    common::assert_valid_markdown(&output);
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
    assert!(first.contains("📦 **CRATE OF THE WEEK** 📦"));
    assert!(first.contains("[demo](https://example.com)"));
    assert!(first.contains("📰 **NEXT** 📰"));
    common::assert_valid_markdown(&first);
}

#[cfg(feature = "integration")]
#[test]
fn telegram_request_sent() {
    run_single_post(
        "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- item\n",
        false,
        true,
    );
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
    run_single_post(
        "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- item\n",
        true,
        false,
    );
}

#[cfg(feature = "integration")]
#[test]
fn sends_valid_markdown() {
    run_single_post(
        "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- **Bold**\n",
        false,
        true,
    );
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
    // Issue 606 currently generates 5 posts, so expect that
    // many requests to the mock server.
    for _ in 0..5 {
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

    for i in 1..=5 {
        let post = fs::read_to_string(dir.path().join(format!("output_{i}.md"))).unwrap();
        common::assert_valid_markdown(&post);
    }
    for m in mocks {
        m.assert();
    }
}

#[cfg(feature = "integration")]
#[test]
fn send_issue_606_post_4() {
    let posts =
        generator::generate_posts(include_str!("2025-07-02-this-week-in-rust.md").to_string())
            .unwrap();
    assert!(posts.len() > 3);
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
        .with_body("{\"ok\":true,\"result\":true}")
        .expect(1)
        .create();

    generator::send_to_telegram(
        &[posts[3].clone()],
        &server.url(),
        "TEST",
        "42",
        true,
        false,
    )
    .unwrap();
    m.assert();
    common::assert_valid_markdown(&posts[3]);
}

#[cfg(feature = "integration")]
#[test]
fn pin_first_message() {
    use mockito::Matcher;

    let posts = vec!["hello".to_string()];
    let mut server = mockito::Server::new();
    let m1 = server
        .mock("POST", "/botTEST/sendMessage")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded("chat_id".into(), "42".into()),
            Matcher::UrlEncoded("parse_mode".into(), "MarkdownV2".into()),
            Matcher::UrlEncoded("disable_web_page_preview".into(), "true".into()),
        ]))
        .with_status(200)
        .with_body("{\"ok\":true,\"result\":{\"message_id\":1}}")
        .expect(1)
        .create();

    let m2 = server
        .mock("POST", "/botTEST/pinChatMessage")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded("chat_id".into(), "42".into()),
            Matcher::UrlEncoded("message_id".into(), "1".into()),
        ]))
        .with_status(200)
        .with_body("{\"ok\":true,\"result\":true}")
        .expect(1)
        .create();

    let m3 = server
        .mock("POST", "/botTEST/deleteMessage")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded("chat_id".into(), "42".into()),
            Matcher::UrlEncoded("message_id".into(), "2".into()),
        ]))
        .with_status(200)
        .with_body("{\"ok\":true,\"result\":true}")
        .expect(1)
        .create();

    generator::send_to_telegram(&posts, &server.url(), "TEST", "42", true, true).unwrap();
    m1.assert();
    m2.assert();
    m3.assert();
    common::assert_valid_markdown(&posts[0]);
}
