# Project Roadmap

This roadmap focuses on transforming "This Week in Rust" posts into valid Telegram messages.

- Parse the Markdown source with `pulldown-cmark` and split it into sections.
- Store each block (headings, lists, paragraphs, code) in dedicated structs for easier conversion.
- Convert those structs to Telegram Markdown using `teloxide` escaping helpers.
- Validate every generated post through `validator::validate_telegram_markdown` before sending.
- Provide thorough unit tests for each component and optional integration tests with Telegram.
- Run `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo machete` in CI to detect issues early.
