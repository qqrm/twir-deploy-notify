# Project Architecture

This tool processes weekly "This Week in Rust" Markdown files and prepares messages for Telegram.

## Structure
- **src/main.rs**: Entry point that calls into the CLI module.
- **src/cli.rs**: Parses command-line arguments and orchestrates the workflow.
- **src/shared/**: Shared implementation of the generator, parser and validator.
- **src/generator.rs**: Re-exports the generator functions.
- **src/parser.rs**: Re-exports the parser implementation.
- **src/validator.rs**: Re-exports the validator logic.
- **Cargo.toml**: Defines dependencies such as `pulldown-cmark` and `teloxide`.
- **last_sent.txt**: Records the last processed issue for the workflow.
- **src/bin/verify_posts.rs**: Sends posts to Telegram and confirms that they
  were delivered correctly.

## Parsing Flow
1. The input Markdown includes metadata lines beginning with `Title:`, `Number:`, and `Date:`.
2. `pulldown-cmark` parses the rest of the file into sections based on `##` headings and list items.
3. Each list item is converted to Telegram Markdown while escaping special characters.
4. Links are preserved using parentheses format, and a final link to the full issue is generated from the date and number.

## Message Generation
- Each section becomes a separate Telegram post capped at 4000 characters.
- Long messages are split by `split_posts`, which scans for escaped characters when breaking lines so that Telegram accepts every chunk. Every post after the first is prefixed with `*Part X/Y*`, where `X` and `Y` are plain digits.
- The issue title in the first post is surrounded by crab emojis.
- The optional `--plain` flag removes Markdown formatting for channels that require plain text.

## Dependencies
- `pulldown-cmark` handles Markdown parsing.
- `teloxide` utilities assist with escaping text for Telegram Markdown.

