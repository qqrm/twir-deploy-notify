# Project Architecture

This tool processes weekly "This Week in Rust" Markdown files and prepares messages for Telegram.

## Structure
- **src/main.rs**: Entry point that calls into the CLI module.
- **src/cli.rs**: Parses command-line arguments and orchestrates the workflow.
- **src/generator.rs**: Builds Telegram posts and sends them to the API.
- **src/parser.rs**: Splits Markdown into sections using `pulldown-cmark`.
- **src/validator.rs**: Checks that generated posts follow Telegram Markdown rules.
- **Cargo.toml**: Defines dependencies such as `pulldown-cmark`, `regex`, and `teloxide`.
- **last_sent.txt**: Records the last processed issue for the workflow.

## Parsing Flow
1. The input Markdown includes metadata lines beginning with `Title:`, `Number:`, and `Date:`.
2. `pulldown-cmark` parses the rest of the file into sections based on `##` headings and list items.
3. Each list item is converted to Telegram Markdown while escaping special characters.
4. Links are preserved using parentheses format, and a final link to the full issue is generated from the date and number.

## Message Generation
- Each section becomes a separate Telegram post capped at 4000 characters.
- Long messages are split by `split_posts`, which scans for escaped characters when breaking lines so that Telegram accepts every chunk. Each post is prefixed with `*Part X/Y*`, where the numbers use emoji digits.
- The optional `--plain` flag removes Markdown formatting for channels that require plain text.

## Dependencies
- `pulldown-cmark` handles Markdown parsing.
- `regex` extracts metadata and processes links.
- `teloxide` utilities assist with escaping text for Telegram Markdown.

