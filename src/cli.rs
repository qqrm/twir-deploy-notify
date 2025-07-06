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

    let input = fs::read_to_string(&cli.input)?;
    let mut posts = generate_posts(input).map_err(|e| std::io::Error::other(e.to_string()))?;

    if cli.plain {
        posts = posts.into_iter().map(|p| markdown_to_plain(&p)).collect();
    }

    write_posts(&posts, Path::new("."))?;
    for (i, _) in posts.iter().enumerate() {
        println!("Generated output_{}.md", i + 1);
    }
    if let (Ok(token), Ok(chat_id)) = (env::var("TELEGRAM_BOT_TOKEN"), env::var("TELEGRAM_CHAT_ID"))
    {
        log::debug!("chat id: {chat_id}");
        let base = env::var("TELEGRAM_API_BASE")
            .unwrap_or_else(|_| "https://api.telegram.org".to_string());
        let pin_first = env::var("TELEGRAM_PIN_FIRST")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        log::info!("calling send_to_telegram");
        send_to_telegram(&posts, &base, &token, &chat_id, !cli.plain, pin_first)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
    }
    Ok(())
}
