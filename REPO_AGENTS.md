# Agent Instructions

* Responses may be in **Russian** or **English**, as appropriate.
* All source code, comments, and documentation must be written in **English**.
* Interpret user requests as tasks and prefer delivering complete solutions (e.g., a pull‑request or full code changes) over short snippets.

## Task Isolation and Parallel Execution

Parallel task execution in a single working directory is forbidden.

- Mandatory model: **1 task = 1 branch = 1 worktree = 1 pull request**.
- Create a dedicated `git worktree` for each new task before making edits.
- Run edits, checks, commits, and pushes only inside the assigned task worktree.
- Keep the primary checkout for repository sync and integration only (`fetch`, `pull`, `merge`, worktree create/remove).
- After task completion and merge, remove the task worktree and delete the task branch when appropriate.

## Rust Build Isolation

When multiple tasks run concurrently, ensure independent Rust build output per worktree:

```bash
export CARGO_TARGET_DIR=.target
```

Do not share a common `CARGO_TARGET_DIR` across concurrent task worktrees.

## Pre‑commit Requirements

Before committing or opening a PR, install and verify:

```bash
rustup component add clippy rustfmt
```

After making changes, ensure all of the following succeed:

1. **Formatting:**

   ```bash
   cargo fmt --all
   ```
2. **Linting:**

   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
3. **Tests:**

   ```bash
   cargo test
   ```
4. **Dependency analysis:**

   ```bash
   cargo machete
   ```

## Documentation and Design

* Review **`DOCS/ARCHITECTURE.md`** and ensure your changes align with the documented design.
* Follow the guidelines in **`DOCS/PARSING.md`**, especially relying on existing crates for Markdown processing rather than custom parsing.
* Remove or feature‑gate any unused functions or dead code.
