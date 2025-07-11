#![cfg(feature = "integration")]
use twir_deploy_notify::generator;

use generator::{generate_posts, send_to_telegram};
use reqwest::blocking::Client;
use serde_json::Value;
use std::env;
mod common;

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
            if let Some(id) = upd["update_id"].as_i64() {
                if id > last_update {
                    last_update = id;
                }
            }
        }
    }

    let input = include_str!("2025-06-25-this-week-in-rust.md");
    let posts = generate_posts(input.to_string()).unwrap();
    let chat_id_num = chat_id.parse::<i64>().ok();
    for p in &posts {
        common::assert_valid_markdown(p);
        send_to_telegram(&[p.clone()], &base, &token, &chat_id, true, false)?;
        let updates_url = format!(
            "{}/bot{}/getUpdates?offset={}",
            base.trim_end_matches('/'),
            token,
            last_update + 1
        );
        let resp: Value = client.get(&updates_url).send()?.json()?;
        let mut last_text = None;
        if let Some(arr) = resp["result"].as_array() {
            for upd in arr {
                if let Some(id) = upd["update_id"].as_i64() {
                    if id > last_update {
                        last_update = id;
                    }
                }
                let msg = upd.get("channel_post").or_else(|| upd.get("message"));
                if let Some(m) = msg
                    && m["chat"]["id"].as_i64() == chat_id_num
                    && let Some(text) = m["text"].as_str()
                {
                    last_text = Some(text.to_string());
                }
            }
        }

        let received = last_text.expect("No message from Telegram");
        assert_eq!(p, &received, "Telegram message mismatch");
    }

    Ok(())
}

#[test]
fn telegram_pin_message() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    let send_url = format!("{}/bot{}/sendMessage", base.trim_end_matches('/'), token);
    let resp: Value = client
        .post(&send_url)
        .form(&[
            ("chat_id", chat_id.as_str()),
            ("text", "pin test"),
            ("parse_mode", "MarkdownV2"),
            ("disable_web_page_preview", "true"),
        ])
        .send()?
        .json()?;
    assert!(
        resp["ok"].as_bool().unwrap_or(false),
        "sendMessage failed: {resp:?}"
    );
    let msg_id = resp["result"]["message_id"]
        .as_i64()
        .ok_or("missing message_id")?
        .to_string();

    std::thread::sleep(std::time::Duration::from_millis(
        generator::TELEGRAM_PIN_DELAY_MS,
    ));

    let pin_url = format!("{}/bot{}/pinChatMessage", base.trim_end_matches('/'), token);
    let pin_resp: Value = client
        .post(&pin_url)
        .form(&[
            ("chat_id", chat_id.as_str()),
            ("message_id", msg_id.as_str()),
        ])
        .send()?
        .json()?;
    assert!(
        pin_resp["ok"].as_bool().unwrap_or(false),
        "pinChatMessage failed: {pin_resp:?}"
    );

    Ok(())
}
