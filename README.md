# postcss-rs

[![CI](https://github.com/njmyers/postcss-rs/actions/workflows/test.yml/badge.svg)](https://github.com/njmyers/postcss-rs/actions/workflows/test.yml)
[![npm](https://img.shields.io/npm/v/@postcss-rs/node-postcss)](https://www.npmjs.com/package/@postcss-rs/node-postcss)
[![license](https://img.shields.io/npm/l/@postcss-rs/node-postcss)](./LICENSE.md)

A drop-in replacement for the PostCSS parser — 100% compatible, ~15% faster than postcss on large CSS.

postcss-rs implements the PostCSS parser in Rust and exposes it to Node.js via NAPI. It returns real PostCSS `Root` instances, so it works with all existing PostCSS plugins and tooling without any migration.

## Benchmark Results

Averages across 5 runs, measured with `pnpm bench` (ops/s, higher is better):

| Mode                     | Small (~200 B) | Medium (~2 KB) | Large (~30 KB) |
| ------------------------ | -------------- | -------------- | -------------- |
| postcss-rs offsets sync  | 63,552         | **9,878**      | **646**        |
| postcss-rs offsets async | 13,472         | 5,455          | 639            |
| postcss-rs json sync     | 33,358         | 5,061          | 305            |
| postcss-rs json async    | 9,978          | 3,309          | 301            |
| postcss original         | **67,571**     | 9,488          | 562            |

The offsets sync mode is the fastest mode and outperforms postcss original on medium and large CSS (~4% and ~15% faster respectively). For small CSS it is ~6% slower due to NAPI call overhead. The json modes carry additional overhead from full AST serialization and are considerably slower.

## Installation

```bash
npm install @postcss-rs/node-postcss
# or
pnpm add @postcss-rs/node-postcss
```

No Rust toolchain required — pre-compiled native binaries are included for all major platforms.

## Quick Start

```typescript
import { parse } from '@postcss-rs/node-postcss';

const root = await parse('.button { color: red; padding: 8px; }');

console.log(root.first.selector); // '.button'
console.log(root.first.first.prop); // 'color'
console.log(root.first.first.value); // 'red'
```

To use with PostCSS plugins, use the default export:

```typescript
import postcssRs from '@postcss-rs/node-postcss';
import autoprefixer from 'autoprefixer';

const result = await postcssRs([autoprefixer]).process(css, {
  from: 'input.css',
});
```

## Documentation

<!-- docs site link will go here once deployed -->

- [Getting Started](packages/docs/guide/getting-started.md)
- [Why postcss-rs?](packages/docs/guide/why.md)
- [Architecture](packages/docs/guide/architecture.md)
- [API Reference](packages/docs/api/index.md)

## License

MIT © Nick Myers
