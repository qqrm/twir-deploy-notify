# Markdown Parsing Guidelines

- Always use established crates like `pulldown-cmark` for reading and parsing Markdown files.
- Avoid implementing Markdown parsing logic manually unless absolutely necessary.
- Review all Markdown files in the repository before starting new tasks, as they may contain important instructions or data for the project.
- Valid Telegram Markdown must be confirmed using an external library such as [`teloxide`](https://crates.io/crates/teloxide).
- The crate currently lacks a dedicated validator, so the project provides a
  minimal implementation in [`src/validator.rs`](src/validator.rs) until such
  functionality becomes available upstream.
