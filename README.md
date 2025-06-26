# TWIR Deploy Notify

This repository contains a small tool and workflow for sending summaries of the latest **This Week in Rust** post to Telegram.

The GitHub Actions workflow checks out the [`rust-lang/this-week-in-rust`](https://github.com/rust-lang/this-week-in-rust) repository and detects the newest Markdown file in its `content` directory. If a new issue is found, it is parsed with the Rust application in `src/main.rs`, and the generated message is posted to the configured Telegram chat. Messages longer than Telegram's limit are split into several posts automatically.
The parser now derives the HTML link from the issue number and date and appends it at the end of each Telegram message.

To run the workflow locally you must clone the `this-week-in-rust` repository into a `twir` subdirectory:

```bash
git clone https://github.com/rust-lang/this-week-in-rust twir
```

After that you can run the tool manually with:

```bash
cargo run -- twir/content/<file-name>.md
```

The workflow keeps track of the last processed file in `last_sent.txt`.
