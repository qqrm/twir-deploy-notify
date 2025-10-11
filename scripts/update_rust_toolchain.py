#!/usr/bin/env python3
"""Refresh the repository's pinned Rust toolchain."""


from __future__ import annotations

import os
import pathlib
import re
import urllib.request

DOC_PATH = pathlib.Path('DOCS/RUST_VERSION.md')
TOOLCHAIN_PATH = pathlib.Path('rust-toolchain.toml')
OUTPUT = pathlib.Path(os.environ['GITHUB_OUTPUT'])
SOURCE_URL = 'https://static.rust-lang.org/dist/channel-rust-stable.toml'


def fetch_channel_data() -> str:
    with urllib.request.urlopen(SOURCE_URL) as response:
        return response.read().decode()


def resolve_latest(metadata: str) -> tuple[str, str]:
    version_match = re.search(r"\[pkg\.rust\]\s+version = \"([^\"]+)\"", metadata)
    date_match = re.search(r"^date\s*=\s*\"([^\"]+)\"", metadata, re.MULTILINE)
    if not version_match:
        raise SystemExit('failed to locate stable toolchain version')
    if not date_match:
        raise SystemExit('failed to locate channel release date')
    latest = version_match.group(1).split()[0]
    release_date = date_match.group(1)
    return latest, release_date


def resolve_current() -> str:
    if TOOLCHAIN_PATH.exists():
        current_match = re.search(
            r'^channel\s*=\s*\"([0-9]+\.[0-9]+\.[0-9]+)\"',
            TOOLCHAIN_PATH.read_text(),
            re.MULTILINE,
        )
        if current_match:
            return current_match.group(1)
    if DOC_PATH.exists():
        current_match = re.search(r'`([0-9]+\.[0-9]+\.[0-9]+)`', DOC_PATH.read_text())
        if current_match:
            return current_match.group(1)
    return ''


def write_outputs(latest: str, release_date: str, current: str, changed: bool) -> None:
    with OUTPUT.open('a', encoding='utf-8') as fh:
        print(f'latest={latest}', file=fh)
        print(f'release_date={release_date}', file=fh)
        print(f'current={current}', file=fh)
        print(f'changed={str(changed).lower()}', file=fh)


def update_files(latest: str, release_date: str) -> None:
    DOC_PATH.write_text(
        '\n'.join(
            (
                '# Rust Toolchain',
                '',
                'The automation tracks the latest stable Rust release used by the workflows.',
                '',
                f'- Version: `{latest}`',
                f'- Source: {SOURCE_URL}',
                f'- Updated: {release_date}',
                '',
            )
        )
    )
    TOOLCHAIN_PATH.write_text(
        '\n'.join(
            (
                '[toolchain]',
                f'channel = "{latest}"',
                'components = ["rustfmt", "clippy"]',
                '',
            )
        )
    )


def main() -> None:
    metadata = fetch_channel_data()
    latest, release_date = resolve_latest(metadata)
    current = resolve_current()
    changed = latest != current
    write_outputs(latest, release_date, current, changed)
    if not changed:
        raise SystemExit(0)
    update_files(latest, release_date)


if __name__ == '__main__':
    main()
