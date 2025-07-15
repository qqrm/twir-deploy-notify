use reqwest::blocking::Client;
use serde_json::Value;
use std::{env, fs};

use twir_deploy_notify::generator::{generate_posts, normalize_chat_id, send_to_telegram};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = std::env::args().nth(1).expect("missing input file");
    let input = fs::read_to_string(path)?;
    let posts = generate_posts(input).map_err(|e| format!("{e}"))?;
    let token = env::var("TELEGRAM_BOT_TOKEN")?;
    let chat_id_raw = env::var("TELEGRAM_CHAT_ID")?;
    let chat_id_norm = normalize_chat_id(&chat_id_raw);
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
    let chat_id_num = chat_id_norm.as_ref().parse::<i64>().ok();
    for p in &posts {
        send_to_telegram(&[p.clone()], &base, &token, &chat_id_raw, true, false)?;
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
                    && (m["chat"]["id"].as_i64() == chat_id_num
                        || m["chat"]["id"].as_str() == Some(chat_id_norm.as_ref()))
                    && let Some(text) = m["text"].as_str()
                {
                    last_text = Some(text.to_string());
                }
            }
        }
        let received = last_text.ok_or("No message from Telegram")?;
        if &received != p {
            return Err("Telegram message mismatch".into());
        }
    }
    Ok(())
}
