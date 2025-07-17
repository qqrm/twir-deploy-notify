# TWIR Deploy Notify

This repository contains a small tool and workflow for sending summaries of the latest **This Week in Rust** post to Telegram.

The GitHub Actions workflow checks out the [`rust-lang/this-week-in-rust`](https://github.com/rust-lang/this-week-in-rust) repository and detects the newest Markdown file in its `content` directory. If a new issue is found, it is parsed with the Rust application in `src/main.rs`, and the generated message is posted to the configured Telegram chat. Each section becomes an individual Telegram post, and sections or overly long lines exceeding Telegram's size limit are automatically split.
The parser now derives the HTML link from the issue number and date and appends it at the end of each Telegram message.
A single GitHub release tagged `latest` always provides the most recent binary.

## Toolchain

The project requires **Rust 1.88.0** and `rustfmt` **1.8.0**. The `rust-toolchain.toml` file pins the channel and lists the mandatory `clippy` and `rustfmt` components so `rustup` always uses the correct versions. All GitHub Actions workflows are configured with the same toolchain version. If these components are not installed automatically, run:

```bash
rustup component add clippy rustfmt
```

To run the workflow locally you must clone the `this-week-in-rust` repository into a `twir` subdirectory:

```bash
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/rust-lang/this-week-in-rust twir
cd twir && git sparse-checkout set content
cd ..
```

After that you can run the tool manually with:

```bash
cargo run --bin twir-deploy-notify -- twir/content/<file-name>.md
```

All files matching `output_*.md` in the current directory are removed before the
new posts are written.

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

- `TELEGRAM_BOT_TOKEN` – bot token for the selected environment.
- `TELEGRAM_CHAT_ID` – identifier of the chat or channel. Numeric IDs are
  automatically prefixed with `-100` when sending requests to Telegram.
- `DEV_BOT_TOKEN` and `DEV_CHAT_ID` – deprecated variables still recognized by
  the CLI for local runs.

The CLI first uses `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID`, which are
provided by the GitHub environment (`dev` or `prod`). If
either variable is unset, it falls back to `DEV_BOT_TOKEN` and `DEV_CHAT_ID` for
backward compatibility. If no valid credentials are found, the program only
writes the generated posts to disk.

The first sent message is automatically pinned, and the service notification is
removed.
The workflow stores the last processed file in `last_sent.txt` as an artifact and downloads it on the next run.

Responses from Telegram are verified with the `verify-posts` binary.
The `prod.yml` workflow runs hourly on the zeroth minute. It first posts to the development chat and, once verified, delivers the release to the main chat. The `retro.yml` workflow builds posts for the last ten issues and uploads
them as artifacts. All posts are parsed at runtime using the shared parser and generator.

## Development

Continuous integration runs `cargo machete` to verify that `Cargo.toml` lists only used dependencies. Run this command locally before opening a pull request.
Install it with `cargo install cargo-machete` if it is not available.

Documentation in `DOCS/` is validated with `cargo run --bin check-docs`, which parses files using [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark).
Generated Telegram posts are verified with the shared `validator` module.
Security checks using `cargo-audit` can be enabled in the same way by setting the `run_audit` input.

### Auto merge

Pull requests targeting the `main` branch are merged automatically once all CI
checks have completed. The `automerge.yml` workflow waits for every check run on
the pull request commit to finish and fails only if a run reports a non-success
conclusion. Checks that are skipped or marked as neutral do not block merging,
so partially skipped pipelines can still be merged.


## License

This project is distributed under the "QQRM LAPOCHKA v1.0 License (AI-first Vibecoder)" in `LICENSE_QQRM_LAPOCHKA`.
Contributors must generate changes via an AI agent.
Manual code submissions may still be humorously called a "skill issue" by the community.
