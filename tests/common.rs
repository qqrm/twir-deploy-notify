pub fn assert_valid_markdown(post: &str) {
    twir_deploy_notify::validator::validate_telegram_markdown(post)
        .unwrap_or_else(|e| panic!("invalid telegram markdown: {e}"));
}
