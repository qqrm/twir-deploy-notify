# Architecture Overview

The crate turns "This Week in Rust" Markdown into Telegram posts.

## Layout
- `src/main.rs` – CLI entrypoint.
- `src/cli.rs` – parses arguments and triggers post generation.
 - `src/shared/` – parser, generator and validator used by the library.
 - `src/generator.rs`, `src/parser.rs`, `src/validator.rs` – thin re-exports of shared modules.
 - `src/bin/verify_posts.rs` – checks posts by sending them to Telegram.
 - `last_sent.txt` – workflow artifact with the last processed issue.

## Processing
1. Markdown files start with `Title:`, `Number:` and `Date:` lines.
2. `pulldown-cmark` splits the rest into sections using `##` headings and list items.
3. Each item becomes Telegram Markdown with special characters escaped.
4. A final link to the web version is derived from the date and number.

## Posts
- Each section forms a post capped at 4000 characters.
- `split_posts` divides long messages and prefixes later posts with `*Part X/Y*`.
- The `--plain` flag strips formatting for plain text destinations.

## Key crates
- `pulldown-cmark` for Markdown parsing.
- `teloxide` and `reqwest` for Telegram interactions.

