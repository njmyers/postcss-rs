# postcss-rs

[![CI](https://github.com/njmyers/postcss-rs/actions/workflows/test.yml/badge.svg)](https://github.com/njmyers/postcss-rs/actions/workflows/test.yml)
[![npm](https://img.shields.io/npm/v/@postcss-rs/node-postcss)](https://www.npmjs.com/package/@postcss-rs/node-postcss)
[![license](https://img.shields.io/npm/l/@postcss-rs/node-postcss)](./LICENSE.md)

A drop-in replacement for the PostCSS parser — 100% compatible, 11–36% faster than postcss.

postcss-rs implements the PostCSS parser in Rust and exposes it to Node.js via NAPI. It returns real PostCSS `Root` instances, so it works with all existing PostCSS plugins and tooling without any migration.

## Benchmark Results

Measured with `pnpm bench` (ops/s, higher is better):

| Mode                     | Small (~200 B) | Medium (~2 KB) | Large (~30 KB) |
| ------------------------ | -------------- | -------------- | -------------- |
| postcss-rs offsets sync  | **69,311**     | **10,789**     | **738**        |
| postcss-rs offsets async | 13,838         | 5,541          | 709            |
| postcss-rs json sync     | 30,960         | 5,015          | 297            |
| postcss-rs json async    | 9,423          | 3,224          | 294            |
| postcss original         | 62,288         | 9,240          | 542            |

The offsets sync mode is consistently the fastest, outperforming postcss original by ~11% on small CSS, ~17% on medium, and ~36% on large. The single-pass Rust parser eliminates the intermediate AST allocation for the offsets path. The json modes carry additional overhead from full AST serialization and are considerably slower.

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
