#[allow(dead_code)]
#[path = "../src/generator.rs"]
mod generator;
#[allow(dead_code)]
#[path = "../src/parser.rs"]
mod parser;
#[allow(dead_code)]
#[path = "../src/validator.rs"]
mod validator;

use generator::{TELEGRAM_LIMIT, split_posts};
use proptest::prelude::*;
use validator::validate_telegram_markdown;

fn arb_dash_boundary() -> impl Strategy<Value = String> {
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
            prop_assert!(validate_telegram_markdown(&p).is_ok());
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn dash_boundary_preserves_escape(text in arb_dash_boundary()) {
        let posts = split_posts(&text, TELEGRAM_LIMIT);
        prop_assert!(!posts.is_empty());
        for p in posts {
            prop_assert!(!p.starts_with('-'));
            prop_assert!(validate_telegram_markdown(&p).is_ok());
        }
    }
}
