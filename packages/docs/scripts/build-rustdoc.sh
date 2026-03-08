#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DOCS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$DOCS_DIR/public/rustdoc"

cd "$REPO_ROOT"

RUSTDOCFLAGS="--extend-css $DOCS_DIR/scripts/rustdoc-custom.css" \
  cargo doc --no-deps -p postcss-rs-parser -p postcss-rs-tokenizer

rm -rf "$DEST"
mkdir -p "$DEST"
cp -r target/doc/. "$DEST/"
