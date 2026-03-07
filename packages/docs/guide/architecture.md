# Architecture

postcss-rs is a hybrid Rust/Node.js library. The CSS parser is implemented in Rust and exposed to Node.js via [NAPI-rs](https://napi.rs/). The output is then hydrated into real PostCSS class instances.

## Parse Pipeline

```
CSS String
    │
    ▼
Rust Parser (postcss-rs-parser)
    │  Tokenizes and parses CSS into an AST
    │
    ▼
PostCSS-compatible JSON
    │  Rust outputs JSON that matches PostCSS's internal format exactly
    │
    ▼
JSON.parse()  (~0.15ms, V8 built-in)
    │
    ▼
postcss.fromJSON()
    │  Hydrates plain objects into Root, Rule, Declaration, etc.
    │
    ▼
PostCSS Root
```

## Rust Parser

The core parser lives in `packages/parser/src/` and is built as a Rust library (`cdylib`). It produces a PostCSS-compatible JSON representation of the CSS AST with:

- **Conditional field serialization** — `nodes` is omitted on leaf nodes (declarations, comments) to match PostCSS's output
- **`inputId` on all nodes** — source objects include `inputId: 0` matching PostCSS's format
- **`inputs` array on root** — the root node carries an `inputs` array with CSS source and file path
- **Correct `source.end`** — root node inherits its end position from the last child

## NAPI Bindings

`packages/node-postcss/src/lib.rs` defines an async NAPI task (`AsyncParseTask`) that:

1. Accepts a CSS string and optional `from` path
2. Runs the Rust parser on a thread pool (non-blocking)
3. Returns the JSON string to JavaScript

The async design ensures the Node.js event loop is never blocked during parsing.

## JavaScript Layer

`packages/node-postcss/src/index.ts` provides the public TypeScript API:

- **`parse(css, opts)`** — async, calls the Rust NAPI binding and hydrates the result
- **`postcssRs(plugins)`** — creates a PostCSS `Processor` with the Rust parser injected

## Monorepo Structure

```
packages/
├── parser/          Rust CSS parser (core)
├── tokenizer/       Rust CSS tokenizer (dependency of parser)
├── node/            NAPI bindings for tokenizer
├── node-postcss/    Main package — Rust + TypeScript public API
├── node-tokenizer/  NAPI bindings for tokenizer only
└── postcss/         Platform-specific npm packages for binary distribution
```

## Why JSON Instead of Direct JS Objects?

The Rust parser returns a JSON string rather than constructing JavaScript objects directly through NAPI. This is intentional:

- V8's `JSON.parse()` is highly optimized and handles typical CSS in ~0.15ms
- Passing structured objects across the NAPI boundary adds per-property overhead that exceeds JSON parsing cost for non-trivial ASTs
- The JSON approach keeps the Rust/JS boundary simple and maintainable
