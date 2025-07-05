#[allow(dead_code)]
#[path = "../src/validator.rs"]
mod validator;
use validator::validate_telegram_markdown;

#[test]
fn dash_inside_word_is_ok() {
    let msg =
        "Always wanted to contribute to open-source projects but did not know where to start?";
    assert!(validate_telegram_markdown(msg).is_ok());
}
