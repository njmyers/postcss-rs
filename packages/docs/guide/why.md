# Why postcss-rs?

PostCSS is the backbone of modern CSS tooling — used by Tailwind, Autoprefixer, CSS Modules, and hundreds of other tools. But its parser is written in JavaScript, which can become a bottleneck in large projects or build pipelines that process many CSS files.

**postcss-rs** replaces PostCSS's parser with a Rust implementation that is 100% compatible with the PostCSS API and plugin ecosystem. Performance on large CSS files is already ~15% ahead of postcss; further improvements are an active area of development.

## The Performance Problem

In a typical PostCSS pipeline, parsing is one of the first and most frequent operations. For a large project with hundreds of CSS files, even a few milliseconds of parsing overhead per file compounds into seconds of build time.

The JavaScript PostCSS parser is well-written, but it faces inherent limitations of the JS runtime:

- Single-threaded execution
- JIT compilation warmup costs
- Garbage collection pauses

## The Rust Solution

Rust compiles to native machine code with no runtime overhead. By implementing the CSS parser in Rust and exposing it to Node.js via [NAPI-rs](https://napi.rs/), postcss-rs aims to achieve:

- **Faster parsing** for typical CSS workloads
- **Identical output** — the Rust parser produces PostCSS-compatible JSON directly

## Benchmark Results

Averages across 5 runs, measured using `benny` (ops/s, higher is better):

| Mode                     | Small (~200 B) | Medium (~2 KB) | Large (~30 KB) |
| ------------------------ | -------------- | -------------- | -------------- |
| postcss-rs offsets sync  | 63,552         | **9,878**      | **646**        |
| postcss-rs offsets async | 13,472         | 5,455          | 639            |
| postcss-rs json sync     | 33,358         | 5,061          | 305            |
| postcss-rs json async    | 9,978          | 3,309          | 301            |
| postcss original         | **67,571**     | 9,488          | 562            |

The offsets sync mode is the fastest mode and outperforms postcss original on medium and large CSS (~4% and ~15% faster respectively). For small CSS it is ~6% slower due to NAPI call overhead. The json modes carry additional overhead from serializing the full AST to a JSON string before parsing it in JavaScript and are considerably slower.

## Full PostCSS Compatibility

postcss-rs is not a fork or a reimplementation of PostCSS — it is a faster parser for the same system. The output is hydrated into real PostCSS class instances using `postcss.fromJSON()`, so you get:

- Real `Root`, `Rule`, `Declaration`, `Comment`, and `AtRule` instances
- Full PostCSS plugin compatibility
- Source maps support
- Identical `source`, `raws`, and `inputs` structure

## When Should You Use It?

postcss-rs is a good fit if:

- You process many CSS files in a build pipeline
- Build performance is important to your team
- You are already using PostCSS and want a drop-in speed improvement

It may not be necessary if:

- You only process a handful of small CSS files
- Your build bottleneck is elsewhere (e.g. JavaScript bundling)
