# TWIR Deploy Notify

TWIR Deploy Notify is a small Rust CLI that turns the latest This Week in Rust issue into a series of Telegram posts.

What it does:

1. Finds the newest TWIR Markdown issue file in `this-week-in-rust/content`.
2. Parses the issue into sections.
3. Generates Telegram friendly messages:
   - one Telegram post per section
   - automatically splits sections and overly long lines to fit Telegram limits
   - appends the issue link (derived from issue number and date) to each post
4. Sends the posts to the configured Telegram chat.
5. Pins the first sent message and removes the service notification.
6. Cleans up any previously generated `output_*.md` files before writing new ones.

## Why it exists

- TWIR issues are long, and Telegram has message size limits.
- You want consistent formatting and splitting without manual editing.
- You want a repeatable way to publish the latest issue to one or more chats.

## Requirements

- Rust toolchain (stable).
- A local clone of `rust-lang/this-week-in-rust` with the `content` directory available.
- Telegram bot token and chat id.

## Setup for local use

Clone TWIR into a `twir` subdirectory (sparse checkout of `content`):

```bash
git clone --depth 1 --filter=blob:none --sparse https://github.com/rust-lang/this-week-in-rust twir && cd twir && git sparse-checkout set content && cd ..
```

## Run

```bash
cargo run --bin twir-deploy-notify -- twir/content/<file-name>.md
```

Generate plain text instead of Telegram markup:

```bash
cargo run --bin twir-deploy-notify -- --plain twir/content/<file-name>.md
```

Enable detailed logs:

```bash
RUST_LOG=info cargo run --bin twir-deploy-notify -- twir/content/<file-name>.md
```

## Configuration

Environment variables for sending to Telegram:

- `DEV_BOT_TOKEN` and `DEV_CHAT_ID` dev chat credentials
- `PROD_BOT_TOKEN` and `PROD_CHAT_ID` production chat credentials
  - If `*_CHAT_ID` is numeric, it is automatically prefixed with `-100` when sending requests to Telegram
- `TWIR_SKIP_DEVELOPER_SEND` optional boolean that skips the developer send
- `TWIR_SKIP_PRODUCTION_SEND` optional boolean that skips the production send

If neither credential pair is fully configured, the CLI exits with an error. Partial configuration (only one variable from a pair) is also treated as an error.

## License

See `LICENSE_QQRM_LAPOCHKA`.
