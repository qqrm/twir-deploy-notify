use clap::Parser as ClapParser;
use std::{
    env::{self, VarError},
    fs, io,
    path::Path,
};

enum CredentialSource {
    Primary,
    Fallback,
}

struct Credentials {
    token: String,
    chat_id: String,
    source: CredentialSource,
}

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

    let skip_developer_send = read_bool_flag("TWIR_SKIP_DEVELOPER_SEND")?;
    let skip_production_send = read_bool_flag("TWIR_SKIP_PRODUCTION_SEND")?;

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

    let developer_credentials = if skip_developer_send {
        log::info!(
            "Developer Telegram send skipped via TWIR_SKIP_DEVELOPER_SEND environment variable",
        );
        None
    } else {
        let creds = read_credentials_pair(
            ("DEV_BOT_TOKEN", "DEV_CHAT_ID"),
            Some(("TELEGRAM_BOT_TOKEN", "TELEGRAM_CHAT_ID")),
            "developer Telegram",
        )?;

        log::debug!("developer chat id: {}", creds.chat_id);
        log::info!("Sending posts to developer Telegram chat");
        send_to_telegram(
            &posts,
            &base,
            &creds.token,
            &creds.chat_id,
            !cli.plain,
            true,
        )
        .map_err(|e| io::Error::other(e.to_string()))?;

        Some(creds)
    };

    if skip_production_send {
        log::info!(
            "Production Telegram send skipped via TWIR_SKIP_PRODUCTION_SEND environment variable",
        );
        return Ok(());
    }

    if developer_credentials
        .as_ref()
        .is_some_and(|creds| matches!(creds.source, CredentialSource::Fallback))
    {
        log::warn!(
            "Developer credentials resolved from TELEGRAM_* variables; production send skipped",
        );
        return Ok(());
    }

    let production_credentials = read_credentials_pair(
        ("TELEGRAM_BOT_TOKEN", "TELEGRAM_CHAT_ID"),
        None,
        "production Telegram",
    )?;

    log::debug!("production chat id: {}", production_credentials.chat_id);
    log::info!("Sending posts to production Telegram chat");
    send_to_telegram(
        &posts,
        &base,
        &production_credentials.token,
        &production_credentials.chat_id,
        !cli.plain,
        true,
    )
    .map_err(|e| io::Error::other(e.to_string()))?;

    Ok(())
}

fn read_optional_env(name: &str) -> io::Result<Option<String>> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
        Ok(_) => {
            log::error!("{name} is empty");
            Err(io::Error::other(format!("{name} is empty")))
        }
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => {
            log::error!("{name} contains invalid UTF-8");
            Err(io::Error::other(format!("{name} contains invalid UTF-8")))
        }
    }
}

fn read_bool_flag(name: &str) -> io::Result<bool> {
    match env::var(name) {
        Ok(value) => {
            if value.trim().is_empty() {
                log::error!("{name} is empty");
                return Err(io::Error::other(format!("{name} is empty")));
            }
            match value.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => Ok(true),
                "0" | "false" | "no" | "off" => Ok(false),
                other => {
                    log::error!("{name} must be a boolean flag; got {other}");
                    Err(io::Error::other(format!(
                        "{name} must be a boolean flag (true/false/1/0)",
                    )))
                }
            }
        }
        Err(VarError::NotPresent) => Ok(false),
        Err(VarError::NotUnicode(_)) => {
            log::error!("{name} contains invalid UTF-8");
            Err(io::Error::other(format!("{name} contains invalid UTF-8")))
        }
    }
}

fn read_credentials_pair(
    primary: (&str, &str),
    fallback: Option<(&str, &str)>,
    label: &str,
) -> io::Result<Credentials> {
    match read_pair(primary)? {
        PairState::Complete(token, chat_id) => Ok(Credentials {
            token,
            chat_id,
            source: CredentialSource::Primary,
        }),
        PairState::Missing => {
            if let Some(names) = fallback {
                match read_pair(names)? {
                    PairState::Complete(token, chat_id) => Ok(Credentials {
                        token,
                        chat_id,
                        source: CredentialSource::Fallback,
                    }),
                    PairState::Missing => {
                        log::error!("{label} credentials not provided");
                        Err(io::Error::other(format!(
                            "{label} credentials not provided; aborting deployment"
                        )))
                    }
                    PairState::Partial => Err(io::Error::other(format!(
                        "{label} credentials incomplete; aborting deployment"
                    ))),
                }
            } else {
                log::error!("{label} credentials not provided");
                Err(io::Error::other(format!(
                    "{label} credentials not provided; aborting deployment"
                )))
            }
        }
        PairState::Partial => Err(io::Error::other(format!(
            "{label} credentials incomplete; aborting deployment"
        ))),
    }
}

enum PairState {
    Complete(String, String),
    Missing,
    Partial,
}

fn read_pair(names: (&str, &str)) -> io::Result<PairState> {
    let first = read_optional_env(names.0)?;
    let second = read_optional_env(names.1)?;

    match (first, second) {
        (Some(token), Some(chat_id)) => Ok(PairState::Complete(token, chat_id)),
        (None, None) => Ok(PairState::Missing),
        (Some(_), None) => {
            log::error!(
                "{} provided without {}; please configure both or unset both",
                names.0,
                names.1
            );
            Ok(PairState::Partial)
        }
        (None, Some(_)) => {
            log::error!(
                "{} provided without {}; please configure both or unset both",
                names.1,
                names.0
            );
            Ok(PairState::Partial)
        }
    }
}
