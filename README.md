# TWIR Deploy Notify

This repository contains a small tool and workflow for sending summaries of the latest **This Week in Rust** post to Telegram.

The GitHub Actions workflow checks out the [`rust-lang/this-week-in-rust`](https://github.com/rust-lang/this-week-in-rust) repository and detects the newest Markdown file in its `content` directory. If a new issue is found, it is parsed with the Rust application in `src/main.rs`, and the generated message is posted to the configured Telegram chat. Each section becomes an individual Telegram post, and sections or overly long lines exceeding Telegram's size limit are automatically split.
The parser now derives the HTML link from the issue number and date and appends it at the end of each Telegram message.
A single GitHub release tagged `latest` always provides the most recent binary.

## Toolchain

All GitHub Actions workflows read the pinned channel from `rust-toolchain.toml` and install the required `clippy` and `rustfmt` components automatically. The scheduled `TWIR Update Rust toolchain` workflow resolves the latest stable release, updates both the manifest and `DOCS/RUST_VERSION.md`, and opens an auto-merged pull request whenever the version changes. This keeps the repository active while ensuring every pipeline runs against the same compiler version. If the components are not installed automatically, run:

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

Pass `--plain` to generate plain text output. Tables are now rendered without
code fences or extra padding by default.

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
- `DEV_BOT_TOKEN` and `DEV_CHAT_ID` – credentials for the developer chat. The
  CLI falls back to `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` when the
  dedicated developer variables are not present, which skips the production
  delivery.
- `TWIR_SKIP_DEVELOPER_SEND` – optional boolean flag (`true`, `false`, `1`,
  `0`, etc.) that skips the developer send. Used by the production workflow
  because it already dispatched a developer run in a separate job.
- `TWIR_SKIP_PRODUCTION_SEND` – optional boolean flag that skips the production
  send. The developer workflow enables it to avoid double-posting when only the
  developer chat should receive messages.

If neither pair is fully configured, the CLI terminates with an error. Partial
configuration (setting only one variable from a pair) is also treated as an
error and stops the run with an actionable log message.

The first sent message is automatically pinned, and the service notification is
removed.
Production runs store the last processed file in `last_sent.txt` as an artifact and download it on the next run. The workflow now fails if this artifact cannot be retrieved.

Telegram acknowledgements are validated directly by the CLI, and the `prod.yml`
workflow waits for the developer delivery job to succeed before the production
run starts. The workflow runs hourly on the zeroth minute and publishes the
latest post directly to the main chat. The `retro.yml` workflow builds posts for the last ten issues and uploads
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
