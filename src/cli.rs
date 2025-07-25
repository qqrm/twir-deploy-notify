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
    // Use main chat credentials if available, otherwise fall back to the
    // development variables. This allows running the debug pipeline without
    // requiring production secrets.
    let token = env::var("TELEGRAM_BOT_TOKEN")
        .or_else(|_| env::var("DEV_BOT_TOKEN"))
        .ok();
    let chat_id = env::var("TELEGRAM_CHAT_ID")
        .or_else(|_| env::var("DEV_CHAT_ID"))
        .ok();

    let token_valid = token.as_ref().is_some_and(|t| !t.trim().is_empty());
    let chat_id_valid = chat_id.as_ref().is_some_and(|c| !c.trim().is_empty());

    if token_valid && chat_id_valid {
        let token_ref = token.as_ref().unwrap();
        let chat_id_ref = chat_id.as_ref().unwrap();
        log::debug!("chat id: {chat_id_ref}");
        let base = env::var("TELEGRAM_API_BASE")
            .unwrap_or_else(|_| "https://api.telegram.org".to_string());
        log::info!("Sending posts to Telegram");
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
        log::warn!("Telegram credentials missing; skipping send");
    }
    Ok(())
}
