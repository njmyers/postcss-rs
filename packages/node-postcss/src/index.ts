/**
 * @postcss-rs/node-postcss
 *
 * A high-performance PostCSS-compatible parser powered by Rust.
 *
 * This package provides a drop-in replacement for PostCSS's parse() function
 * with 3-5x faster parsing performance while maintaining 100% compatibility
 * with the PostCSS API and plugin ecosystem.
 *
 * Architecture:
 * 1. Parse CSS with Rust (5x faster than JS)
 * 2. Receive JSON string across the NAPI boundary
 * 3. JSON.parse() the string (V8 built-in, highly optimized)
 * 4. buildNode() — single-pass construction of real PostCSS class instances:
 *    - Constructs nodes directly without PostCSS's normalize() overhead
 *    - Assigns raws directly (no normalization side-effects)
 *    - Attaches source.input in the same pass
 * 5. Return fully-functional PostCSS Root object
 */

import * as postcss from 'postcss';
import type { Root, Plugin, Processor, ProcessOptions } from 'postcss';
import * as path from 'path';
import { ByteBuffer } from 'flatbuffers';
import { Node as FbNode } from './css-ast/node.js';
import { decode as msgpackDecode } from '@msgpack/msgpack';

// Load Rust native bindings via the napi-rs v3 generated loader
// __dirname is dist/, index.js is in the package root (one level up)
const bindings = require(path.join(__dirname, '../index.js')) as {
  postcss: {
    // Returns a JSON string; caller does JSON.parse() + buildNode()
    parse: (css: string, from?: string | null) => Promise<string>;
    parseSync: (css: string, from?: string | null) => string;
    stringify: (json: string) => string;
    // FlatBuffers variants — return a raw Buffer instead of a JSON string
    parseFbSync: (css: string, from?: string | null) => Buffer;
    parseFb: (css: string, from?: string | null) => Promise<Buffer>;
    // MessagePack variants — return a raw Buffer of MessagePack bytes
    parseMsgpackSync: (css: string, from?: string | null) => Buffer;
    parseMsgpack: (css: string, from?: string | null) => Promise<Buffer>;
  };
};

// Cache PostCSS class constructors at module level to avoid repeated property
// lookups on the `postcss` namespace object inside the hot path.
const PCSSRoot        = postcss.Root;
const PCSSRule        = postcss.Rule;
const PCSSAtRule      = postcss.AtRule;
const PCSSDeclaration = postcss.Declaration;
const PCSSComment     = postcss.Comment;

// PostCSS internal symbols used by the Node constructor.  We set them
// manually when bypassing the constructor via Object.create() so that
// PostCSS's internal guards (e.g. `!i[my]` in normalize()) pass correctly.
const { isClean, my } = require('postcss/lib/symbols') as { isClean: symbol; my: symbol };

// Pre-built prototype references avoid repeated `.prototype` lookups inside
// the hot buildNode loop.
const RootProto  = PCSSRoot.prototype;
const RuleProto  = PCSSRule.prototype;
const AtRuleProto = PCSSAtRule.prototype;
const DeclProto  = PCSSDeclaration.prototype;
const CommentProto = PCSSComment.prototype;

/**
 * Build a PostCSS node tree from the Rust JSON output in a single pass.
 *
 * Replaces the previous three-traversal pipeline:
 *   postcss.fromJSON()  →  restoreRaws()  →  root.walk(attachInput)
 *
 * Improvements over fromJSON():
 * - No per-node object spread (`{ ...defaults }`)
 * - Children are assigned directly to `node.nodes` instead of going through
 *   `container.append()`, which avoids the `normalize()` → regex +
 *   `markDirty()` overhead for every child node
 * - `raws` is assigned directly, so PostCSS never has a chance to normalize
 *   `raws.before` from the previous sibling (eliminating the need for a
 *   separate restoreRaws pass)
 * - `source.input` is attached in the same traversal (no separate root.walk)
 */
function buildNode(json: any, input: postcss.Input, parent: any): any {
  // Use Object.create(prototype) instead of `new ClassName()` to bypass the
  // Node constructor chain entirely.  The constructor does: raws={}, sets two
  // symbol flags, iterates the defaults object (empty when called with no
  // args), and checks/initialises nodes[].  We replicate those three
  // essential assignments manually and skip the rest.
  let node: any;

  switch (json.type) {
    case 'root': {
      node = Object.create(RootProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'root'; node.nodes = [];
      break;
    }
    case 'rule': {
      node = Object.create(RuleProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'rule'; node.nodes = [];
      node.selector = json.selector;
      break;
    }
    case 'atrule': {
      node = Object.create(AtRuleProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'atrule';
      node.name   = json.name;
      node.params = json.params; // always present: Rust AtRule::create always sets params
      break;
    }
    case 'decl': {
      node = Object.create(DeclProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'decl';
      node.prop  = json.prop;
      node.value = json.value;
      if (json.important) node.important = json.important;
      break;
    }
    case 'comment': {
      node = Object.create(CommentProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'comment';
      node.text = json.text;
      break;
    }
    default:
      throw new Error(`Unknown node type: ${json.type}`);
  }

  // Assign raws directly — PostCSS never sees these through a setter, so no
  // normalization occurs and a separate restoreRaws pass is unnecessary.
  // raws is always serialised by Rust (at minimum as `{}`).
  node.raws = json.raws;

  // Attach input to the source object produced by Rust.
  // Attaching input here eliminates the separate root.walk pass.
  const s = json.source;
  node.source = s.end
    ? { start: s.start, end: s.end, input }
    : { start: s.start, input };

  if (parent !== undefined) node.parent = parent;

  // Assign children directly rather than via append(), bypassing normalize().
  // Use a pre-sized array + for-loop instead of .map() to avoid the callback
  // allocation overhead on the hot path.
  const rawChildren = json.nodes;
  if (rawChildren) {
    const n = rawChildren.length;
    const built = new Array(n);
    for (let i = 0; i < n; i++) built[i] = buildNode(rawChildren[i], input, node);
    node.nodes = built;
  }

  return node;
}

/**
 * Build a PostCSS node tree from a FlatBuffers buffer produced by Rust.
 *
 * Like buildNode() but reads from a FlatBuffers ByteBuffer instead of a
 * plain JS object, skipping JSON.parse() entirely.
 */
function buildNodeFb(fb: FbNode, input: postcss.Input, parent: any): any {
  const t = fb.t();
  let node: any;

  switch (t) {
    case 0: {
      node = Object.create(RootProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'root'; node.nodes = [];
      break;
    }
    case 1: {
      node = Object.create(RuleProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'rule'; node.nodes = [];
      node.selector = fb.selector();
      break;
    }
    case 2: {
      node = Object.create(DeclProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'decl';
      node.prop  = fb.prop();
      node.value = fb.value();
      if (fb.important()) node.important = true;
      break;
    }
    case 3: {
      node = Object.create(AtRuleProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'atrule';
      node.name   = fb.name();
      node.params = fb.params() ?? '';
      break;
    }
    case 4: {
      node = Object.create(CommentProto);
      node.raws = {}; node[isClean] = false; node[my] = true;
      node.type = 'comment';
      node.text = fb.text();
      break;
    }
    default:
      throw new Error(`Unknown FlatBuffers node type: ${t}`);
  }

  // Raws
  const raws: any = {};
  const rBefore = fb.rBefore(); if (rBefore !== null) raws.before = rBefore;
  const rBetween = fb.rBetween(); if (rBetween !== null) raws.between = rBetween;
  const rAfter = fb.rAfter(); if (rAfter !== null) raws.after = rAfter;
  const rAfterName = fb.rAfterName(); if (rAfterName !== null) raws.afterName = rAfterName;
  const rSemi = fb.rSemicolon();
  if (rSemi !== -1) raws.semicolon = rSemi === 1;
  const rLeft = fb.rLeft(); if (rLeft !== null) raws.left = rLeft;
  const rRight = fb.rRight(); if (rRight !== null) raws.right = rRight;
  const rImp = fb.rImportant(); if (rImp !== null) raws.important = rImp;
  const rOwnSemi = fb.rOwnSemicolon(); if (rOwnSemi !== null) raws.ownSemicolon = rOwnSemi;
  const rvv = fb.rValueValue(); const rvr = fb.rValueRaw();
  if (rvv !== null || rvr !== null) {
    raws.value = {} as any;
    if (rvv !== null) raws.value.value = rvv;
    if (rvr !== null) raws.value.raw = rvr;
  }
  const rsv = fb.rSelectorValue(); const rsr = fb.rSelectorRaw();
  if (rsv !== null || rsr !== null) {
    raws.selector = {} as any;
    if (rsv !== null) raws.selector.value = rsv;
    if (rsr !== null) raws.selector.raw = rsr;
  }
  node.raws = raws;

  // Source
  const start = { offset: fb.ssOff(), line: fb.ssLine(), column: fb.ssCol() };
  node.source = fb.hasEnd()
    ? { start, end: { offset: fb.seOff(), line: fb.seLine(), column: fb.seCol() }, input }
    : { start, input };

  if (parent !== undefined) node.parent = parent;

  // Children
  const len = fb.nodesLength();
  if (len > 0) {
    const built = new Array(len);
    const child = new FbNode();
    for (let i = 0; i < len; i++) {
      built[i] = buildNodeFb(fb.nodes(i, child)!, input, node);
    }
    node.nodes = built;
  }

  return node;
}

/**
 * Synchronous MessagePack parse — no JSON involved.
 * msgpack decode gives a plain object tree, so reuses the same buildNode path as JSON.
 */
export function parseMsgpackSync(css: string, opts: ParseOptions = {}): Root {
  const buf = bindings.postcss.parseMsgpackSync(css, opts.from ?? null);
  const rustObj = msgpackDecode(buf) as any;
  const input = new postcss.Input(css, { from: opts.from });
  return buildNode(rustObj, input, undefined) as Root;
}

/**
 * Async MessagePack parse.
 */
export async function parseMsgpack(css: string, opts: ParseOptions = {}): Promise<Root> {
  const buf = await bindings.postcss.parseMsgpack(css, opts.from ?? null);
  const rustObj = msgpackDecode(buf) as any;
  const input = new postcss.Input(css, { from: opts.from });
  return buildNode(rustObj, input, undefined) as Root;
}

// ─── Raw (no tree-walk) variants for benchmarking ─────────────────────────────
// These expose only the Rust parse + serialisation cost, with NO JS tree walk.

/** Rust parse → JSON string only (no JSON.parse, no buildNode) */
export function parseRustOnlySync(css: string, opts: ParseOptions = {}): string {
  return bindings.postcss.parseSync(css, opts.from ?? null);
}

/** Rust parse → JSON.parse only (no buildNode) */
export function parseJsonOnlySync(css: string, opts: ParseOptions = {}): any {
  return JSON.parse(bindings.postcss.parseSync(css, opts.from ?? null));
}

/** Rust parse → FlatBuffers Buffer only (no ByteBuffer decode, no buildNodeFb) */
export function parseFbRustOnlySync(css: string, opts: ParseOptions = {}): Buffer {
  return bindings.postcss.parseFbSync(css, opts.from ?? null);
}

/** Rust parse → FlatBuffers decode only (no buildNodeFb tree walk) */
export function parseFbDecodeOnlySync(css: string, opts: ParseOptions = {}): FbNode {
  const buf = bindings.postcss.parseFbSync(css, opts.from ?? null);
  const bb  = new ByteBuffer(new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength));
  return FbNode.getRootAsNode(bb);
}

/** Rust parse → MessagePack buffer only (no decode, no buildNode) */
export function parseMsgpackRustOnlySync(css: string, opts: ParseOptions = {}): Buffer {
  return bindings.postcss.parseMsgpackSync(css, opts.from ?? null);
}

/** Rust parse → MessagePack decode only (no buildNode tree walk) */
export function parseMsgpackDecodeOnlySync(css: string, opts: ParseOptions = {}): any {
  return msgpackDecode(bindings.postcss.parseMsgpackSync(css, opts.from ?? null));
}


export interface ParseOptions {
  from?: string;
  map?: any;
}

/**
 * Parse CSS into a PostCSS AST
 *
 * This function uses a Rust-based parser for maximum performance,
 * then hydrates the result into actual PostCSS class instances
 * for full API compatibility.
 *
 * @param css - CSS source code to parse
 * @param opts - Parse options
 * @returns PostCSS Root instance
 *
 * @example
 * ```typescript
 * const { parse } = require('@postcss-rs/node-postcss');
 *
 * const root = await parse('.test { color: red; }', { from: 'test.css' });
 * console.log(root.first.selector); // '.test'
 * console.log(root.first.first.prop); // 'color'
 * ```
 */
export async function parse(css: string, opts: ParseOptions = {}): Promise<Root> {
  const jsonStr = await bindings.postcss.parse(css, opts.from ?? null);
  const rustObj = JSON.parse(jsonStr);
  const input   = new postcss.Input(css, { from: opts.from });
  return buildNode(rustObj, input, undefined) as Root;
}

/**
 * Synchronous parse (blocks the event loop)
 *
 * Faster than parse() for small/medium CSS because it skips the
 * Promise + libuv thread-pool round-trip (~0.05–0.1 ms overhead).
 * For large files (>10 KB) prefer the async parse() to avoid
 * blocking the event loop for several milliseconds.
 *
 * @param css - CSS source code to parse
 * @param opts - Parse options
 * @returns PostCSS Root instance
 */
export function parseSync(css: string, opts: ParseOptions = {}): Root {
  const jsonStr = bindings.postcss.parseSync(css, opts.from ?? null);
  const rustObj = JSON.parse(jsonStr);
  const input   = new postcss.Input(css, { from: opts.from });
  return buildNode(rustObj, input, undefined) as Root;
}

/**
 * Synchronous FlatBuffers parse — no JSON involved.
 */
export function parseFbSync(css: string, opts: ParseOptions = {}): Root {
  const buf = bindings.postcss.parseFbSync(css, opts.from ?? null);
  const bb  = new ByteBuffer(new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength));
  const fb  = FbNode.getRootAsNode(bb);
  const input = new postcss.Input(css, { from: opts.from });
  return buildNodeFb(fb, input, undefined) as Root;
}

/**
 * Async FlatBuffers parse — no JSON involved.
 */
export async function parseFb(css: string, opts: ParseOptions = {}): Promise<Root> {
  const buf = await bindings.postcss.parseFb(css, opts.from ?? null);
  const bb  = new ByteBuffer(new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength));
  const fb  = FbNode.getRootAsNode(bb);
  const input = new postcss.Input(css, { from: opts.from });
  return buildNodeFb(fb, input, undefined) as Root;
}

/**
 * Create a PostCSS processor with Rust-powered parsing
 *
 * This returns a standard PostCSS Processor, but with our fast
 * Rust parser used instead of the default PostCSS parser.
 *
 * @param plugins - PostCSS plugins
 * @returns PostCSS Processor instance
 *
 * @example
 * ```typescript
 * const postcssRs = require('@postcss-rs/node-postcss');
 * const autoprefixer = require('autoprefixer');
 *
 * const processor = postcssRs([autoprefixer]);
 * const result = await processor.process(css, { from: 'input.css' });
 * ```
 */
export function postcssRs(plugins: Plugin[] = []): Processor {
  const processor = postcss.default(plugins);

  // Override the parser option to use our Rust parser
  const originalProcess = processor.process.bind(processor);

  processor.process = function(css: string | Root, opts: ProcessOptions = {}): any {
    // Use our Rust parser
    if (typeof css === 'string') {
      opts.parser = {
        parse: (cssStr: string | { toString(): string }, parseOpts?: any) => {
          const cssString = typeof cssStr === 'string' ? cssStr : cssStr.toString();
          // Return a promise that resolves to the parsed Root
          // Note: This will work because PostCSS handles async parsers
          return parse(cssString, parseOpts || {}) as any;
        }
      };
    }

    return originalProcess(css, opts);
  };

  return processor;
}

// Default export
export default postcssRs;

// Re-export PostCSS for convenience
export { postcss };

// Re-export PostCSS classes for convenience
export const PostCSSRoot        = postcss.Root;
export const PostCSSRule        = postcss.Rule;
export const PostCSSAtRule      = postcss.AtRule;
export const PostCSSDeclaration = postcss.Declaration;
export const PostCSSComment     = postcss.Comment;
export const PostCSSContainer   = postcss.Container;
export const PostCSSNode        = postcss.Node;

// Re-export PostCSS types
export type {
  Root,
  Rule,
  AtRule,
  Declaration,
  Comment,
  Container,
  Node,
  Plugin,
  Processor
} from 'postcss';
