use clap::Parser as ClapParser;
use std::{env, fs, io, path::Path};

use crate::generator::{generate_posts, markdown_to_plain, send_to_telegram, write_posts};

#[derive(ClapParser)]
struct Cli {
    /// Input Markdown file
    input: String,

    /// Generate plain text output
    #[arg(long)]
    plain: bool,
}

/// Entry point for the command-line interface.
///
/// Reads the provided Markdown file, generates Telegram posts, optionally
/// converts them to plain text and sends them to Telegram if credentials are
/// available.
pub fn main() -> std::io::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    log::info!("Reading input file {}", cli.input);
    let input = fs::read_to_string(&cli.input)?;
    log::info!("Generating posts");
    let mut posts = generate_posts(input).map_err(|e| std::io::Error::other(e.to_string()))?;
    log::info!("Generated {} posts", posts.len());

    if cli.plain {
        log::info!("Converting posts to plain text");
        posts = posts.into_iter().map(|p| markdown_to_plain(&p)).collect();
    }

    log::info!("Writing posts to disk");
    write_posts(&posts, Path::new("."))?;

    let base =
        env::var("TELEGRAM_API_BASE").unwrap_or_else(|_| "https://api.telegram.org".to_string());

    let dev_token = env::var("DEV_BOT_TOKEN").ok();
    let dev_chat_id = env::var("DEV_CHAT_ID").ok();
    let dev_token_valid = dev_token.as_ref().is_some_and(|t| !t.trim().is_empty());
    let dev_chat_id_valid = dev_chat_id.as_ref().is_some_and(|c| !c.trim().is_empty());

    let mut dev_sent = false;

    if dev_token_valid && dev_chat_id_valid {
        let token_ref = dev_token.as_ref().unwrap();
        let chat_id_ref = dev_chat_id.as_ref().unwrap();
        log::debug!("developer chat id: {chat_id_ref}");
        log::info!("Sending posts to developer Telegram chat");
        send_to_telegram(&posts, &base, token_ref, chat_id_ref, !cli.plain, true)
            .map_err(|e| io::Error::other(e.to_string()))?;
        dev_sent = true;
    } else {
        match dev_token {
            Some(ref t) if t.trim().is_empty() => log::error!("DEV_BOT_TOKEN is empty"),
            None => log::error!("DEV_BOT_TOKEN not set"),
            _ => {}
        }
        match dev_chat_id {
            Some(ref c) if c.trim().is_empty() => log::error!("DEV_CHAT_ID is empty"),
            None => log::error!("DEV_CHAT_ID not set"),
            _ => {}
        }
        log::warn!("Developer Telegram credentials missing; skipping developer send");
    }

    if dev_sent {
        let token = env::var("TELEGRAM_BOT_TOKEN").ok();
        let chat_id = env::var("TELEGRAM_CHAT_ID").ok();
        let token_valid = token.as_ref().is_some_and(|t| !t.trim().is_empty());
        let chat_id_valid = chat_id.as_ref().is_some_and(|c| !c.trim().is_empty());

        if token_valid && chat_id_valid {
            let token_ref = token.as_ref().unwrap();
            let chat_id_ref = chat_id.as_ref().unwrap();
            log::debug!("production chat id: {chat_id_ref}");
            log::info!("Sending posts to production Telegram chat");
            send_to_telegram(&posts, &base, token_ref, chat_id_ref, !cli.plain, true)
                .map_err(|e| io::Error::other(e.to_string()))?;
        } else {
            match token {
                Some(ref t) if t.trim().is_empty() => log::error!("TELEGRAM_BOT_TOKEN is empty"),
                None => log::error!("TELEGRAM_BOT_TOKEN not set"),
                _ => {}
            }
            match chat_id {
                Some(ref c) if c.trim().is_empty() => log::error!("TELEGRAM_CHAT_ID is empty"),
                None => log::error!("TELEGRAM_CHAT_ID not set"),
                _ => {}
            }
            log::warn!("Production Telegram credentials missing; skipping production send");
        }
    } else {
        log::warn!(
            "Developer Telegram send failed or was skipped; production send will not be attempted"
        );
    }
    Ok(())
}
