# Agent Instructions

* Responses may be in **Russian** or **English**, as appropriate.
* All source code, comments, and documentation must be written in **English**.
* Interpret user requests as tasks and prefer delivering complete solutions (e.g., a pull‑request or full code changes) over short snippets.

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
