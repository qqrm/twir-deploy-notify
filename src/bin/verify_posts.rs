use reqwest::blocking::Client;
use serde_json::Value;
use std::{env, fs, thread, time::Duration};

use twir_deploy_notify::generator::{generate_posts, normalize_chat_id, send_to_telegram};

const FETCH_RETRIES: usize = 5;
const FETCH_DELAY_MS: u64 = 1_000;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = std::env::args().nth(1).expect("missing input file");
    let input = fs::read_to_string(path)?;
    let posts = generate_posts(input).map_err(|e| format!("{e}"))?;
    let token = env::var("TELEGRAM_BOT_TOKEN").map_err(|_| "TELEGRAM_BOT_TOKEN not set")?;
    if token.trim().is_empty() {
        return Err("TELEGRAM_BOT_TOKEN is empty".into());
    }
    let chat_id_raw = env::var("TELEGRAM_CHAT_ID").map_err(|_| "TELEGRAM_CHAT_ID not set")?;
    if chat_id_raw.trim().is_empty() {
        return Err("TELEGRAM_CHAT_ID is empty".into());
    }
    let chat_id_norm = normalize_chat_id(&chat_id_raw);
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
    let chat_id_num = chat_id_norm.as_ref().parse::<i64>().ok();
    for (idx, post) in posts.iter().enumerate() {
        let single = &posts[idx..idx + 1];
        send_to_telegram(single, &base, &token, &chat_id_raw, true, false)?;
        let mut received = None;

        for attempt in 0..FETCH_RETRIES {
            if attempt > 0 {
                thread::sleep(Duration::from_millis(FETCH_DELAY_MS));
            }

            let updates_url = format!(
                "{}/bot{}/getUpdates?offset={}",
                base.trim_end_matches('/'),
                token,
                last_update + 1
            );
            let resp: Value = client.get(&updates_url).send()?.json()?;

            if let Some(arr) = resp["result"].as_array() {
                for upd in arr {
                    if let Some(id) = upd["update_id"].as_i64()
                        && id > last_update
                    {
                        last_update = id;
                    }

                    let msg = upd.get("channel_post").or_else(|| upd.get("message"));
                    if let Some(m) = msg
                        && (m["chat"]["id"].as_i64() == chat_id_num
                            || m["chat"]["id"].as_str() == Some(chat_id_norm.as_ref()))
                        && let Some(text) = m["text"].as_str()
                    {
                        received = Some(text.to_string());
                    }
                }
            }

            if received.is_some() {
                break;
            }
        }

        let received = received
            .ok_or_else(|| format!("No message from Telegram after {} attempts", FETCH_RETRIES))?;
        if received != *post {
            return Err("Telegram message mismatch".into());
        }
    }
    Ok(())
}
