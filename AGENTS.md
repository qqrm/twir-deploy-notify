# AGENT Instructions

- Responses may be in Russian or English as appropriate.
- All code comments and technical documentation must be written in English.
- Install required Rust components with `rustup component add clippy rustfmt`.
- After making any changes, run `cargo fmt --all`, `cargo check --all-targets --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.
- Fix every issue reported by these commands before committing or submitting pull requests.
- A pull request is complete only when formatting, linting, `cargo check`, and tests all succeed.
- Always review `DEVLOG.md` and `ARCHITECTURE.md` before making any modifications.
- Configure the remote `origin` as `https://github.com/qqrm/twir-deploy-notify`.
- Before beginning or finalizing work on a task, run `git fetch origin` and
  check whether `origin/main` contains new commits. Rebase your branch onto the
  latest `origin/main` if needed so all development starts from the most recent
  commit.
