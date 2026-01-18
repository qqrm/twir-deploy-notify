use reqwest::blocking::Client;
use serde_json::Value;
use std::{collections::VecDeque, env, fs, thread, time::Duration};

use twir_deploy_notify::generator::{generate_posts, normalize_chat_id};

const MAX_ATTEMPTS: usize = 10;
const POLL_DELAY: Duration = Duration::from_secs(2);

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = std::env::args().nth(1).expect("missing input file");
    let input = fs::read_to_string(path)?;
    let posts = generate_posts(input).map_err(|e| format!("{e}"))?;
    if posts.is_empty() {
        println!("No posts generated; skipping Telegram verification");
        return Ok(());
    }
    let (token, chat_id_raw) = read_credentials()?;
    let chat_id_norm = normalize_chat_id(&chat_id_raw);
    let chat_username = chat_id_raw
        .trim()
        .strip_prefix('@')
        .map(|name| name.to_ascii_lowercase());
    let base =
        env::var("TELEGRAM_API_BASE").unwrap_or_else(|_| "https://api.telegram.org".to_string());
    let client = Client::new();
    let chat_id_num = chat_id_norm.as_ref().parse::<i64>().ok();
    let mut window: VecDeque<String> = VecDeque::with_capacity(posts.len());
    let mut last_update = None;
    let mut offset = None;

    for attempt in 0..MAX_ATTEMPTS {
        let updates_url = match offset {
            Some(next) => format!(
                "{}/bot{}/getUpdates?offset={}",
                base.trim_end_matches('/'),
                token,
                next
            ),
            None => format!("{}/bot{}/getUpdates", base.trim_end_matches('/'), token),
        };
        let resp: Value = client.get(&updates_url).send()?.json()?;
        let mut saw_update = false;

        if let Some(arr) = resp["result"].as_array() {
            for upd in arr {
                if let Some(id) = upd["update_id"].as_i64() {
                    saw_update = true;
                    if last_update.is_none_or(|prev| id > prev) {
                        last_update = Some(id);
                    }
                }

                let msg = upd.get("channel_post").or_else(|| upd.get("message"));
                if let Some(m) = msg {
                    let chat = &m["chat"];
                    let id_matches = chat_id_num
                        .is_some_and(|expected| chat["id"].as_i64() == Some(expected))
                        || chat["id"].as_str() == Some(chat_id_norm.as_ref());
                    let username_matches = chat_username.as_ref().is_some_and(|expected| {
                        chat["username"]
                            .as_str()
                            .map(|found| found.eq_ignore_ascii_case(expected))
                            .unwrap_or(false)
                    });

                    if (id_matches || username_matches)
                        && let Some(text) = m["text"].as_str()
                    {
                        window.push_back(text.to_string());
                        while window.len() > posts.len() {
                            window.pop_front();
                        }
                    }
                }
            }
        }

        if window.len() == posts.len() && window.iter().eq(posts.iter()) {
            if let Some(id) = last_update {
                let ack_url = format!(
                    "{}/bot{}/getUpdates?offset={}",
                    base.trim_end_matches('/'),
                    token,
                    id + 1
                );
                let _ = client.get(&ack_url).send();
            }
            return Ok(());
        }

        offset = last_update.map(|id| id + 1);

        if !saw_update && attempt + 1 == MAX_ATTEMPTS {
            break;
        }

        thread::sleep(POLL_DELAY);
    }

    if window.is_empty() {
        return Err("No message from Telegram".into());
    }

    let preview: Vec<String> = window
        .iter()
        .map(|text| text.chars().take(50).collect())
        .collect();
    Err(format!(
        "Telegram message mismatch; latest posts: {}",
        preview.join(" | ")
    )
    .into())
}

fn read_credentials() -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(pair) = read_pair(("DEV_BOT_TOKEN", "DEV_CHAT_ID"))? {
        return Ok(pair);
    }
    if let Some(pair) = read_pair(("PROD_BOT_TOKEN", "PROD_CHAT_ID"))? {
        return Ok(pair);
    }
    Err("DEV_BOT_TOKEN/DEV_CHAT_ID or PROD_BOT_TOKEN/PROD_CHAT_ID not set".into())
}

fn read_pair(
    names: (&str, &str),
) -> Result<Option<(String, String)>, Box<dyn std::error::Error + Send + Sync>> {
    let token = env::var(names.0).ok().map(|value| value.trim().to_string());
    let chat_id = env::var(names.1).ok().map(|value| value.trim().to_string());

    match (token, chat_id) {
        (Some(token), Some(chat_id)) => {
            if token.is_empty() {
                return Err(format!("{} is empty", names.0).into());
            }
            if chat_id.is_empty() {
                return Err(format!("{} is empty", names.1).into());
            }
            Ok(Some((token, chat_id)))
        }
        (None, None) => Ok(None),
        (Some(_), None) => Err(format!(
            "{} provided without {}; please configure both",
            names.0, names.1
        )
        .into()),
        (None, Some(_)) => Err(format!(
            "{} provided without {}; please configure both",
            names.1, names.0
        )
        .into()),
    }
}
