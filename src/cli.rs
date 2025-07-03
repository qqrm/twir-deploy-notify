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

pub fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let input = fs::read_to_string(&cli.input)?;
    let mut posts = generate_posts(input);

    if cli.plain {
        posts = posts.into_iter().map(|p| markdown_to_plain(&p)).collect();
    }

    write_posts(&posts, Path::new("."))?;
    for (i, _) in posts.iter().enumerate() {
        println!("Generated output_{}.md", i + 1);
    }
    if let (Ok(token), Ok(chat_id)) = (env::var("TELEGRAM_BOT_TOKEN"), env::var("TELEGRAM_CHAT_ID"))
    {
        let base = env::var("TELEGRAM_API_BASE")
            .unwrap_or_else(|_| "https://api.telegram.org".to_string());
        send_to_telegram(&posts, &base, &token, &chat_id)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
    }
    Ok(())
}
