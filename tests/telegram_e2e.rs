#![cfg(feature = "integration")]

mod common;
#[allow(dead_code)]
#[path = "../src/generator.rs"]
mod generator;
#[allow(dead_code)]
#[path = "../src/parser.rs"]
mod parser;
#[allow(dead_code)]
#[path = "../src/validator.rs"]
mod validator;

use common::assert_valid_markdown;
use generator::{generate_posts, send_to_telegram};
use reqwest::blocking::Client;
use serde_json::Value;
use std::env;

#[test]
fn telegram_end_to_end() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let token = match env::var("TELEGRAM_BOT_TOKEN") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Missing TELEGRAM_BOT_TOKEN, skipping");
            return Ok(());
        }
    };
    let chat_id = match env::var("TELEGRAM_CHAT_ID") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Missing TELEGRAM_CHAT_ID, skipping");
            return Ok(());
        }
    };
    let base =
        env::var("TELEGRAM_API_BASE").unwrap_or_else(|_| "https://api.telegram.org".to_string());

    let client = Client::new();
    let updates_url = format!("{}/bot{}/getUpdates", base.trim_end_matches('/'), token);
    let resp: Value = client.get(&updates_url).send()?.json()?;
    let mut last_update = 0i64;
    if let Some(arr) = resp["result"].as_array() {
        for upd in arr {
            if let Some(id) = upd["update_id"].as_i64()
                && id > last_update
            {
                last_update = id;
            }
        }
    }

    let input = include_str!("2025-06-25-this-week-in-rust.md");
    let posts = generate_posts(input.to_string()).unwrap();
    for p in &posts {
        assert_valid_markdown(p);
    }
    send_to_telegram(&posts, &base, &token, &chat_id, true)?;

    let updates_url = format!(
        "{}/bot{}/getUpdates?offset={}",
        base.trim_end_matches('/'),
        token,
        last_update + 1
    );
    let resp: Value = client.get(&updates_url).send()?.json()?;
    let mut received = Vec::new();
    if let Some(arr) = resp["result"].as_array() {
        for upd in arr {
            let msg = upd.get("channel_post").or_else(|| upd.get("message"));
            if let Some(m) = msg
                && m["chat"]["id"].as_i64() == chat_id.parse::<i64>().ok()
                && let Some(text) = m["text"].as_str()
            {
                received.push(text.to_string());
            }
        }
    }

    assert_eq!(posts, received, "Telegram messages mismatch");
    Ok(())
}
