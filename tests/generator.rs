#[allow(unused_imports)]
use twir_deploy_notify::{generator, parser, validator};

use generator::{TELEGRAM_LIMIT, split_posts};
use proptest::prelude::*;
use validator::validate_telegram_markdown;

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
            prop_assert!(validate_telegram_markdown(&p).is_ok());
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
        validate_telegram_markdown(&p).unwrap();
    }
}
