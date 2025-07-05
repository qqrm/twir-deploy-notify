use twir_deploy_notify::validator::validate_telegram_markdown;

#[test]
fn dash_at_start_is_invalid() {
    assert!(validate_telegram_markdown("- example").is_err());
}
