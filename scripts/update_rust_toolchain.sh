#!/usr/bin/env bash
# Refresh the repository's pinned Rust toolchain.

set -euo pipefail

DOC_PATH="DOCS/RUST_VERSION.md"
TOOLCHAIN_PATH="rust-toolchain.toml"
OUTPUT="${GITHUB_OUTPUT:?GITHUB_OUTPUT is required}"

resolve_latest() {
  local metadata
  metadata=$(rustc +stable --version --verbose)

  local latest
  local release_date
  latest=$(awk '/^release:/ { print $2 }' <<<"$metadata")
  release_date=$(awk '/^commit-date:/ { print $2 }' <<<"$metadata")

  if [[ -z "$latest" ]]; then
    echo "failed to locate stable toolchain version" >&2
    exit 1
  fi

  if [[ -z "$release_date" ]]; then
    echo "failed to locate channel release date" >&2
    exit 1
  fi

  printf '%s\n' "$latest" "$release_date"
}

resolve_current() {
  local current=""
  if [[ -f "$TOOLCHAIN_PATH" ]]; then
    current=$(sed -n 's/^channel[[:space:]]*=[[:space:]]*"\([0-9]\+\.[0-9]\+\.[0-9]\+\)".*/\1/p' "$TOOLCHAIN_PATH" | head -n 1)
  fi
  if [[ -z "$current" && -f "$DOC_PATH" ]]; then
    current=$(sed -n 's/.*`\([0-9]\+\.[0-9]\+\.[0-9]\+\)`.*/\1/p' "$DOC_PATH" | head -n 1)
  fi
  printf '%s\n' "$current"
}

write_outputs() {
  local latest="$1"
  local release_date="$2"
  local current="$3"
  local changed="$4"

  {
    printf 'latest=%s\n' "$latest"
    printf 'release_date=%s\n' "$release_date"
    printf 'current=%s\n' "$current"
    printf 'changed=%s\n' "$changed"
  } >> "$OUTPUT"
}

update_files() {
  local latest="$1"
  local release_date="$2"

  cat <<DOC > "$DOC_PATH"
# Rust Toolchain

The automation tracks the latest stable Rust release used by the workflows.

- Version: \`$latest\`
- Updated: $release_date
DOC

  cat <<TOOLCHAIN > "$TOOLCHAIN_PATH"
[toolchain]
channel = "$latest"
components = ["rustfmt", "clippy"]
TOOLCHAIN
}

main() {
  local latest
  local release_date
  read -r latest release_date < <(resolve_latest)

  local current
  current=$(resolve_current)

  local changed="false"
  if [[ "$latest" != "$current" ]]; then
    changed="true"
  fi

  write_outputs "$latest" "$release_date" "$current" "$changed"

  if [[ "$changed" != "true" ]]; then
    exit 0
  fi

  update_files "$latest" "$release_date"
}

main "$@"
