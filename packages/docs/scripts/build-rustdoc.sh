#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DOCS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$DOCS_DIR/rust-api"

cd "$REPO_ROOT"

cargo +nightly rustdoc -p postcss-rs-parser \
  --output-format json \
  -Zunstable-options \
  2>&1 | grep -v "^warning"

cargo +nightly rustdoc -p postcss-rs-tokenizer \
  --output-format json \
  -Zunstable-options \
  2>&1 | grep -v "^warning"

node "$DOCS_DIR/scripts/rustdoc-to-md.mjs" \
  "$REPO_ROOT/target/doc/postcss_rs_parser.json" \
  "$OUT_DIR" \
  "parser"

node "$DOCS_DIR/scripts/rustdoc-to-md.mjs" \
  "$REPO_ROOT/target/doc/postcss_rs_tokenizer.json" \
  "$OUT_DIR" \
  "tokenizer"
