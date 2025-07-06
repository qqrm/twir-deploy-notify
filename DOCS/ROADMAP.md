# Project Roadmap

This document outlines the major ongoing directions for development.

- Replace unnecessary uses of `regex` with simpler code where possible.
- Expand tests to cover Markdown edge cases.
- Improve CI checks, including a `cargo machete` step.
- Plan and evaluate potential new features.
- Precompute message conversion during compilation by parsing the Markdown
  file at build time so the final binary already contains ready-to-send posts.
