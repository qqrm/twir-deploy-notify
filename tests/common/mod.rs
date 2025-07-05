/// Helper that panics if the provided Telegram Markdown is invalid.
pub fn assert_valid_markdown(post: &str) {
    crate::validator::validate_telegram_markdown(post)
        .unwrap_or_else(|e| panic!("invalid Telegram Markdown: {e}"));
}
