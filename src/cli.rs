use clap::Parser as ClapParser;
use std::{env, fs, path::Path};

use crate::generator::{generate_posts, markdown_to_plain, send_to_telegram, write_posts};

fn read_env_var(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(v) if !v.trim().is_empty() => Some(v),
        _ => {
            log::warn!("{name} not set or empty; skipping send");
            None
        }
    }
}

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
/// set.
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
    let token = read_env_var("TELEGRAM_BOT_TOKEN");
    let chat_id = read_env_var("TELEGRAM_CHAT_ID");
    if let (Some(token), Some(chat_id)) = (token, chat_id) {
        log::debug!("chat id: {chat_id}");
        let base = env::var("TELEGRAM_API_BASE")
            .unwrap_or_else(|_| "https://api.telegram.org".to_string());
        log::info!("Sending posts to Telegram");
        send_to_telegram(&posts, &base, &token, &chat_id, !cli.plain, true)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
    } else {
        log::info!("Skipping Telegram delivery due to missing credentials");
    }
    Ok(())
}
