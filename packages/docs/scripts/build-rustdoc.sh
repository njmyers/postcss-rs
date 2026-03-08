#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DEST="$(cd "$(dirname "$0")/.." && pwd)/.vitepress/public/rustdoc"

cd "$REPO_ROOT"
cargo doc --no-deps -p postcss-rs-parser -p postcss-rs-tokenizer

rm -rf "$DEST"
mkdir -p "$DEST"
cp -r target/doc/. "$DEST/"
