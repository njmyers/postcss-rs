# Getting Started

## Installation

```bash
npm install @postcss-rs/node-postcss
# or
pnpm add @postcss-rs/node-postcss
# or
yarn add @postcss-rs/node-postcss
```

> **Note:** postcss-rs ships pre-compiled native binaries for Linux (x64, ARM64), macOS (x64, ARM64), and Windows (x64, ARM64). No Rust toolchain is required to use the package.

## Basic Usage

### Parsing CSS

```typescript
import { parse } from '@postcss-rs/node-postcss';

const root = await parse('.button { color: red; padding: 8px; }');

console.log(root.first.selector); // '.button'
console.log(root.first.first.prop); // 'color'
console.log(root.first.first.value); // 'red'
```

### With a File Path

Providing the `from` option enables accurate source maps:

```typescript
const root = await parse(css, { from: 'src/styles/button.css' });
```

### Using with PostCSS Plugins

Use `postcssRs()` to create a PostCSS processor backed by the Rust parser:

```typescript
import postcssRs from '@postcss-rs/node-postcss';
import autoprefixer from 'autoprefixer';

const processor = postcssRs([autoprefixer]);

const result = await processor.process(css, { from: 'input.css' });
console.log(result.css);
```

### Using parse() Directly in PostCSS

You can also pass `parse` directly to PostCSS as a custom parser:

```typescript
import postcss from 'postcss';
import { parse } from '@postcss-rs/node-postcss';
import autoprefixer from 'autoprefixer';

const result = await postcss([autoprefixer]).process(css, {
  from: 'input.css',
  parser: { parse },
});
```

## Supported Platforms

| Platform      | x64 | ARM64 | ARM |
| ------------- | --- | ----- | --- |
| Linux (glibc) | ✓   | ✓     | ✓   |
| Linux (musl)  | ✓   | ✓     | —   |
| macOS         | ✓   | ✓     | —   |
| Windows       | ✓   | ✓     | —   |
| FreeBSD       | ✓   | —     | —   |
| Android       | —   | ✓     | —   |
