# TWIR Deploy Notify

This repository contains a small tool and workflow for sending summaries of the latest **This Week in Rust** post to Telegram.

The GitHub Actions workflow checks out the [`rust-lang/this-week-in-rust`](https://github.com/rust-lang/this-week-in-rust) repository and detects the newest Markdown file in its `content` directory. If a new issue is found, it is parsed with the Rust application in `src/main.rs`, and the generated message is posted to the configured Telegram chat. Each section becomes an individual Telegram post, and sections or overly long lines exceeding Telegram's size limit are automatically split.
The parser now derives the HTML link from the issue number and date and appends it at the end of each Telegram message.

## Toolchain

The project requires **Rust 1.88.0** and `rustfmt` **1.8.0**. The `rust-toolchain.toml` file pins the channel and lists the mandatory `clippy` and `rustfmt` components so `rustup` always uses the correct versions. If these components are not installed automatically, run:

```bash
rustup component add clippy rustfmt
```

To run the workflow locally you must clone the `this-week-in-rust` repository into a `twir` subdirectory:

```bash
git clone https://github.com/rust-lang/this-week-in-rust twir
```

After that you can run the tool manually with:

```bash
cargo run --bin twir-deploy-notify -- twir/content/<file-name>.md
```

Set `RUST_LOG=info` to see detailed logs including Telegram API responses:

```bash
RUST_LOG=info cargo run --bin twir-deploy-notify -- twir/content/<file-name>.md
```

Running the workflow with [`act`](https://github.com/nektos/act) is possible, but it requires Docker.
Restricted environments such as the provided container may not support Docker, so executing the above
`cargo` commands manually remains the recommended approach.

The workflow stores the last processed file in `last_sent.txt` as an artifact and downloads it on the next run.

The Telegram API response is checked with `jq`, and the workflow fails if the server does not return `{ "ok": true }`.

## Development

Continuous integration runs `cargo machete` to verify that `Cargo.toml` lists only used dependencies. Run this command locally before opening a pull request.
Install it with `cargo install cargo-machete` if it is not available.

Documentation Markdown is validated with `cargo run --bin check-docs`, which parses files using [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark).
Generated Telegram posts are verified with the shared `validator` module.
