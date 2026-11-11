# postcss-rs

[![CI](https://github.com/njmyers/postcss-rs/actions/workflows/test.yml/badge.svg)](https://github.com/njmyers/postcss-rs/actions/workflows/test.yml)
[![license](https://img.shields.io/badge/license-MIT-blue)](./LICENSE.md)

> **Educational project.** postcss-rs is a learning exercise: a rewrite of the [PostCSS](https://github.com/postcss/postcss) parser in Rust, built to explore Rust, parser design, and Node.js native addons. It is not published to npm and is not maintained for production use. For real projects, use [PostCSS](https://postcss.org).

postcss-rs implements the PostCSS parser in Rust and exposes it to Node.js via [NAPI-RS](https://napi.rs). It returns real PostCSS `Root` instances, so the result can be passed to existing PostCSS plugins and tooling. The parser is tested against PostCSS's own parser test suite, [`postcss-parser-tests`](https://github.com/postcss/postcss-parser-tests).

## What's in the repo

- `packages/tokenizer`: CSS tokenizer
- `packages/parser`: single-pass parser that builds PostCSS-compatible nodes
- `packages/stringifier`: turns parsed nodes back into CSS
- `packages/postcss`: the Node.js package (`@postcss-rs/postcss`), with NAPI bindings, TypeScript types, tests, and benchmarks
- `packages/docs`: guides and API reference

## Benchmarks

Measured locally with the package's benchmark script (ops/s, higher is better). Results will vary by machine.

| Mode                     | Small (~200 B) | Medium (~2 KB) | Large (~30 KB) |
| ------------------------ | -------------- | -------------- | -------------- |
| postcss-rs offsets sync  | **73,789**     | **11,102**     | **777**        |
| postcss-rs offsets async | 13,744         | 5,545          | 721            |
| postcss-rs json sync     | 34,658         | 5,148          | 307            |
| postcss-rs json async    | 10,287         | 3,311          | 299            |
| postcss original         | 69,799         | 9,499          | 558            |

In these runs, offsets sync mode beat the original PostCSS parser by ~6% on small CSS, ~17% on medium, and ~39% on large. The single-pass Rust parser skips the intermediate AST allocation on the offsets path. The json modes carry extra overhead from full AST serialization and are considerably slower.

## Building from source

Requires Node.js 20.12+, pnpm 8+, and a Rust toolchain (version pinned in `rust-toolchain.toml`).

```bash
git clone https://github.com/njmyers/postcss-rs.git
cd postcss-rs
pnpm install
pnpm build:release
pnpm test
pnpm --filter @postcss-rs/postcss bench
```

## Usage

```typescript
import { parseOffsetsSync } from '@postcss-rs/postcss';

const root = parseOffsetsSync('.button { color: red; padding: 8px; }');

console.log(root.first.selector); // '.button'
console.log(root.first.first.prop); // 'color'
console.log(root.first.first.value); // 'red'
```

To run PostCSS plugins, use the default export:

```typescript
import postcssRs from '@postcss-rs/postcss';
import autoprefixer from 'autoprefixer';

const result = await postcssRs([autoprefixer]).process(css, {
  from: 'input.css',
});
```

## Documentation

- [Why postcss-rs?](packages/docs/guide/why.md)
- [Architecture](packages/docs/guide/architecture.md)
- [API Reference](packages/docs/api/index.md)

## Credits

This project reimplements the parser from [PostCSS](https://github.com/postcss/postcss) by Andrey Sitnik and contributors (MIT license), and tests against their [`postcss-parser-tests`](https://github.com/postcss/postcss-parser-tests) suite. All credit for PostCSS's design and API belongs to that project.

## License

MIT © Nick Myers
