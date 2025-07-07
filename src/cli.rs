use clap::Parser as ClapParser;
use std::{env, fs, path::Path};

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
    for (i, _) in posts.iter().enumerate() {
        println!("Generated output_{}.md", i + 1);
    }
    if let (Ok(token), Ok(chat_id)) = (env::var("TELEGRAM_BOT_TOKEN"), env::var("TELEGRAM_CHAT_ID"))
    {
        log::debug!("chat id: {chat_id}");
        let base = env::var("TELEGRAM_API_BASE")
            .unwrap_or_else(|_| "https://api.telegram.org".to_string());
        log::info!("Sending posts to Telegram");
        send_to_telegram(&posts, &base, &token, &chat_id, !cli.plain, true)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
    } else {
        log::info!("TELEGRAM_BOT_TOKEN or TELEGRAM_CHAT_ID not set; skipping send");
    }
    Ok(())
}
