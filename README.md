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

## Cargo profiles

The project defines custom profiles in `Cargo.toml` that optimize for quick
iteration. Development and test builds use `opt-level = 0` with incremental
compilation enabled. Release builds set `opt-level = 1`, keep incremental
compilation, and increase `codegen-units` to `16`.

The `.cargo/config.toml` file adds `-C target-cpu=native` to all builds so local
compilations target the current machine's CPU features.

## Configuration

The application expects several environment variables when sending posts to
Telegram:

- `TELEGRAM_BOT_TOKEN` – the bot token used for authentication.
- `TELEGRAM_CHAT_ID` – the identifier of the chat or channel.
- `TELEGRAM_PIN_FIRST` – set to `1` or `true` to pin the first sent message.
  The service message about the pin will be deleted automatically.

Running the workflow with [`act`](https://github.com/nektos/act) is possible, but it requires Docker.
Restricted environments such as the provided container may not support Docker, so executing the above
`cargo` commands manually remains the recommended approach.

The workflow stores the last processed file in `last_sent.txt` as an artifact and downloads it on the next run.

The Telegram API response is checked with `jq`, and the workflow fails if the server does not return `{ "ok": true }`.

Scheduled runs first send the posts to the development chat using the
`verify-posts` binary. After the messages are confirmed to appear in the
channel, the same release is posted to the main chat.

Setting the `TWIR_MARKDOWN` environment variable before building will
parse the referenced file at compile time and embed the generated posts
in the crate. The resulting array is available as `twir_deploy_notify::posts::POSTS`.

## Development

Continuous integration runs `cargo machete` to verify that `Cargo.toml` lists only used dependencies. Run this command locally before opening a pull request.
Install it with `cargo install cargo-machete` if it is not available.

Documentation Markdown is validated with `cargo run --bin check-docs`, which parses files using [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark).
Generated Telegram posts are verified with the shared `validator` module.
Integration tests that send messages to Telegram run only when the CI workflow is manually triggered with the `run_integration` input.

### Running integration tests

The integration suite relies on the [`mockito`](https://crates.io/crates/mockito) crate to mock network requests.
To exercise the Telegram end‑to‑end test, export the following environment variables before running the tests:

```bash
export TELEGRAM_BOT_TOKEN=<token>
export TELEGRAM_CHAT_ID=<chat id>
cargo test --features integration
```

If these variables are absent, the Telegram tests are skipped.

## Restart command

To restart a task, use the `Restart` command. The agent duplicates the original
task description and prepares a **task stub** that starts from the freshest
commit on `main`. A prompt asks whether to launch the stub as a new merge
request, avoiding stale branches. See [RESTART.md](RESTART.md) for details.

## Commit message template

Git can automatically include the required co-author line in every commit. Set
the template once using:

```bash
git config commit.template .gitmessage
```

Adjust the agent name or email by editing `.gitmessage` directly or by setting
`GIT_AUTHOR_NAME` and `GIT_AUTHOR_EMAIL` before committing.

## License

This project is distributed under the "QQRM LAPOCHKA v1.0 License (AI-first Vibecoder)" in `LICENSE_QQRM_LAPOCHKA`.
Contributors must generate changes via an AI agent and mention it as a co-author in commits.
Manual code submissions may still be humorously called a "skill issue" by the community.
