# Development Log

## 2025-06-26
- Initial project setup and scheduled GitHub Actions workflow.
- Implemented basic Telegram notifier and bug fixes.

## 2025-07-01
- Added Markdown converter and plain text output option.
- Switched parsing to `pulldown-cmark` and filtered HTML comments.

## 2025-07-02
- Expanded unit tests and continuous integration workflow.

## 2025-07-03
- Updated deploy workflow to skip previous version checks when the run is not triggered by the scheduler.
- Fixed message splitting logic to respect Telegram limits and updated integration tests.
- Enhanced post splitting to cut within overly long lines while preserving escapes.

## 2025-07-05
- Identified an issue with `split_posts` cutting lines after escape characters.
- Added regression tests in `tests/generator.rs` verifying correct line splitting.

## 2025-07-06
- Documented local `cargo-machete` installation in README.

## 2025-07-07
- Telegram integration tests are no longer executed automatically.
- They can be run manually by dispatching the CI workflow with `run_integration` set to `true`.

## 2025-07-08
- Removed extraneous separator line from web link section and updated tests.
- Added Rust Jobs chat and feed links to the generated Jobs section.
- Updated tests and expected outputs accordingly.
## 2025-07-09
- Introduced AI-only license and updated contributor guidelines.
## 2025-07-10
- Replaced AI-only license with QQRM LAPOCHKA and updated docs.
## 2025-07-11
- Expanded QQRM LAPOCHKA license text and clarified contributor rules.

## Maintenance
The development log keeps only the 20 most recent entries.
When adding a new entry, delete the oldest if there are already 20.
