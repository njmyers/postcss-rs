# API Reference

All exports are from `@postcss-rs/node-postcss`.

## `parse(css, opts?)`

Parses a CSS string and returns a PostCSS `Root` instance.

```typescript
function parse(css: string, opts?: ParseOptions): Promise<Root>
```

**Parameters**

| Name | Type | Description |
|---|---|---|
| `css` | `string` | CSS source to parse |
| `opts.from` | `string` (optional) | Source file path, used for source maps |
| `opts.map` | `any` (optional) | Source map options (passed through to PostCSS) |

**Returns** `Promise<Root>` — a PostCSS [`Root`](https://postcss.org/api/#root) instance.

**Example**

```typescript
import { parse } from '@postcss-rs/node-postcss'

const root = await parse('.btn { color: blue; }', { from: 'styles.css' })
root.walkDecls(decl => {
  console.log(decl.prop, decl.value)
})
```

---

## `postcssRs(plugins?)`

Creates a PostCSS `Processor` that uses the Rust parser.

```typescript
function postcssRs(plugins?: Plugin[]): Processor
```

**Parameters**

| Name | Type | Description |
|---|---|---|
| `plugins` | `Plugin[]` (optional) | PostCSS plugins to apply |

**Returns** A PostCSS `Processor` instance with the Rust parser injected.

**Example**

```typescript
import postcssRs from '@postcss-rs/node-postcss'
import autoprefixer from 'autoprefixer'

const processor = postcssRs([autoprefixer])
const result = await processor.process(css, { from: 'input.css' })
```

---

## `parseSync(css, opts?)`

> **Not yet implemented.** Calling this will throw an error. Use the async `parse()` instead.

```typescript
function parseSync(css: string, opts?: ParseOptions): Root
```

---

## Re-exports

For convenience, the package re-exports PostCSS classes and types:

```typescript
import {
  postcss,
  PostCSSRoot,
  PostCSSRule,
  PostCSSAtRule,
  PostCSSDeclaration,
  PostCSSComment,
  PostCSSContainer,
  PostCSSNode
} from '@postcss-rs/node-postcss'

// Types
import type {
  Root, Rule, AtRule, Declaration, Comment,
  Container, Node, Plugin, Processor
} from '@postcss-rs/node-postcss'
```

---

## `ParseOptions`

```typescript
interface ParseOptions {
  from?: string  // Source file path for source maps
  map?: any      // Source map options
}
```
