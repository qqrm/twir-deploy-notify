use twir_deploy_notify::generator;

use generator::{TELEGRAM_LIMIT, split_posts};
use proptest::prelude::*;
mod common;

fn arb_dash_boundary_short() -> impl Strategy<Value = String> {
    let prefix_re = format!(r"[A-Za-z0-9]{{{}}}", TELEGRAM_LIMIT - 1);
    proptest::string::string_regex(&prefix_re)
        .unwrap()
        .prop_flat_map(|pre| {
            proptest::string::string_regex("[A-Za-z0-9]{0,10}")
                .unwrap()
                .prop_map(move |post| format!("{pre}\\-{post}"))
        })
}

fn arb_long_line() -> impl Strategy<Value = String> {
    let regex = format!(
        r"[A-Za-z0-9\\*_]{{{},{}}}",
        TELEGRAM_LIMIT * 2,
        TELEGRAM_LIMIT * 3
    );
    proptest::string::string_regex(&regex).unwrap()
}

fn arb_escaped_text() -> impl Strategy<Value = String> {
    let lower = TELEGRAM_LIMIT - 5;
    let upper = TELEGRAM_LIMIT + 5;
    let regex = format!(r"(?:[A-Za-z]|\\\\[-!#\\.]){{{lower},{upper}}}");
    proptest::string::string_regex(&regex).unwrap()
}

fn arb_special_line() -> impl Strategy<Value = String> {
    let word = proptest::string::string_regex("[A-Za-z0-9]{1,5}").unwrap();
    let seq = prop_oneof![Just("\\".to_string()), Just("\\-".to_string()),];
    let word2 = proptest::string::string_regex("[A-Za-z0-9]{0,5}").unwrap();
    (word, prop::collection::vec(seq, 1..3), word2)
        .prop_map(|(pre, seqs, post)| format!("{pre}{}{post}", seqs.concat()))
}

fn arb_multiline_special() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_special_line(), 2..5).prop_map(|lines| lines.join("\n"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn split_posts_handles_long_lines(lines in prop::collection::vec(arb_long_line(), 1..3)) {
        let input = lines.join("\n");
        let posts = split_posts(&input, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
        for p in posts {
            prop_assert!(p.len() <= TELEGRAM_LIMIT);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn escaped_chunks_are_valid(line in arb_escaped_text()) {
        let posts = split_posts(&line, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
        for p in posts {
            common::assert_valid_markdown(&p);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn dash_boundary_preserves_escape(text in arb_dash_boundary_short()) {
        let posts = split_posts(&text, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn dash_escape_at_boundary(input in arb_dash_boundary_short()) {
        let posts = split_posts(&input, TELEGRAM_LIMIT);
        prop_assert!(posts.len() >= 2);
        for p in posts {
            prop_assert!(!p.starts_with('-'));
            common::assert_valid_markdown(&p);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn special_sequences_are_valid(input in arb_multiline_special()) {
        let posts = split_posts(&input, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
        for p in posts {
            prop_assert!(!p.is_empty());
            common::assert_valid_markdown(&p);
        }
    }
}

#[test]
fn boundary_escape_preserved() {
    let mut input = "a".repeat(TELEGRAM_LIMIT - 1);
    input.push('\\');
    input.push_str("-b");
    let posts = split_posts(&input, TELEGRAM_LIMIT);
    assert!(posts.len() >= 2);
    assert!(!posts[1].starts_with('-'));
    for p in posts {
        common::assert_valid_markdown(&p);
    }
}

#[test]
fn single_section_has_expected_prefix() {
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## News\n- item\n";
    let posts = generator::generate_posts(input.to_string()).unwrap();
    assert_eq!(posts.len(), 1);
    assert!(!posts[0].starts_with("*Part"));
}

#[test]
fn quote_section_spacing() {
    let posts =
        generator::generate_posts(include_str!("2025-07-02-this-week-in-rust.md").to_string())
            .unwrap();
    assert!(posts.len() >= 5);
    let lines: Vec<_> = posts[4].lines().collect();
    let idx = lines
        .iter()
        .position(|l| l.contains("Quote of the Week"))
        .unwrap();
    assert!(lines.get(idx + 1).is_some_and(|l| l.is_empty()));
    assert!(lines.get(idx + 2).is_some_and(|l| !l.is_empty()));
    let author_idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with('–'))
        .unwrap();
    assert!(lines.get(author_idx + 1).is_some_and(|l| l.is_empty()));
}

#[test]
fn jobs_url_simplified() {
    let input = "Title: Test\nNumber: 1\nDate: 2025-01-01\n\n## Jobs\nPlease see the latest [Who's Hiring thread on r/rust](https://example.com/thread)\n";
    let posts = generator::generate_posts(input.to_string()).unwrap();
    let combined = posts.join("\n");
    assert!(combined.contains("[Rust Job Reddit Thread](https://example.com/thread)"));
}
