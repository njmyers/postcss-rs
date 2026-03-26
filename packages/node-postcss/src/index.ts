/**
 * @postcss-rs/node-postcss
 *
 * A high-performance PostCSS-compatible parser powered by Rust.
 *
 * Two parse paths are available:
 *
 *   parseSync / parse  — Rust parse → JSON → JSON.parse → buildNode
 *   parseOffsetsSync   — Rust parse → flat Uint32Array of byte offsets →
 *                        buildNodeFromOffsets (css.slice for all strings)
 *
 * The offset path is the default and fastest for large CSS (>10 KB).
 */

import * as postcss from 'postcss';
import type { Root, Plugin, Processor, ProcessOptions } from 'postcss';
import * as path from 'path';

// Load Rust native bindings via the napi-rs v3 generated loader
// __dirname is dist/, index.js is in the package root (one level up)
const bindings = require(path.join(__dirname, '../index.js')) as {
  postcss: {
    // Returns a JSON string; caller does JSON.parse() + buildNode()
    parse: (css: string, from?: string | null) => Promise<string>;
    parseSync: (css: string, from?: string | null) => string;
    stringify: (json: string) => string;
    // Offset-array variant — flat Uint32Array of CSS byte offsets + string table
    parseOffsetsSync: (css: string, from?: string | null) => Buffer;
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
const RootProto    = PCSSRoot.prototype;
const RuleProto    = PCSSRule.prototype;
const AtRuleProto  = PCSSAtRule.prototype;
const DeclProto    = PCSSDeclaration.prototype;
const CommentProto = PCSSComment.prototype;

/**
 * Build a PostCSS node tree from the Rust JSON output in a single pass.
 */
function buildNode(json: any, input: postcss.Input, parent: any): any {
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
      node.params = json.params;
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

  node.raws = json.raws;

  const s = json.source;
  node.source = s.end
    ? { start: s.start, end: s.end, input }
    : { start: s.start, input };

  if (parent !== undefined) node.parent = parent;

  const rawChildren = json.nodes;
  if (rawChildren) {
    const n = rawChildren.length;
    const built = new Array(n);
    for (let i = 0; i < n; i++) built[i] = buildNode(rawChildren[i], input, node);
    node.nodes = built;
  }

  return node;
}

// ─── Offset-array builder ─────────────────────────────────────────────────────

const STRIDE   = 25;
const ABSENT32 = 0xFFFF_FFFF;

const FLAG_IMPORTANT       = 0x01;
const FLAG_SEMI_PRESENT    = 0x02;
const FLAG_SEMI_VALUE      = 0x04;
const FLAG_HAS_CLEAN_STR   = 0x08;
const FLAG_HAS_NODES       = 0x20;

/**
 * Build a PostCSS node tree from the compact offset-array buffer produced
 * by `parseOffsetsSync`.
 *
 * Buffer layout: [u32 node_count] [node_count × STRIDE u32s] [UTF-8 string table]
 *
 * Every string field is recovered with `css.slice(start, end)` — no JSON
 * parsing, no per-string heap copies beyond what V8 already interns.
 */
function buildNodeFromOffsets(buf: Buffer, css: string, input: postcss.Input): Root {
  // Align-safe Uint32Array view (host is LE on all Node.js platforms we target)
  let view: Uint32Array;
  if (buf.byteOffset % 4 === 0) {
    view = new Uint32Array(buf.buffer, buf.byteOffset, buf.byteLength >>> 2);
  } else {
    const copy = new Uint8Array(buf.byteLength);
    copy.set(new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength));
    view = new Uint32Array(copy.buffer);
  }

  const nodeCount  = view[0];
  const strTableBase = (1 + nodeCount * STRIDE) * 4;
  const nodes: any[] = new Array(nodeCount);

  for (let i = 0; i < nodeCount; i++) {
    const o = 1 + i * STRIDE;

    const type      = view[o];
    const parentIdx = view[o + 1];
    const ssOff     = view[o + 2];
    const ssLine    = view[o + 3];
    const ssCol     = view[o + 4];
    const seOff     = view[o + 5];
    const seLine    = view[o + 6];
    const seCol     = view[o + 7];
    const f1s       = view[o + 8];
    const f1e       = view[o + 9];
    const f2s       = view[o + 10];
    const bfS       = view[o + 12];
    const bfE       = view[o + 13];
    const btS       = view[o + 14];
    const btE       = view[o + 15];
    const afS       = view[o + 16];
    const afE       = view[o + 17];
    const ex1S      = view[o + 18];
    const ex1E      = view[o + 19];
    const ex2S      = view[o + 20];
    const ex2E      = view[o + 21];
    const flags     = view[o + 22];
    const clnS      = view[o + 23];
    const clnE      = view[o + 24];

    const start = { offset: ssOff, line: ssLine, column: ssCol };
    const source: any = seOff !== ABSENT32
      ? { start, end: { offset: seOff, line: seLine, column: seCol }, input }
      : { start, input };

    const raws: any = {};
    if (bfS !== ABSENT32) raws.before  = css.slice(bfS, bfE);
    if (btS !== ABSENT32) raws.between = css.slice(btS, btE);
    if (afS !== ABSENT32) raws.after   = css.slice(afS, afE);
    if (flags & FLAG_SEMI_PRESENT) raws.semicolon = (flags & FLAG_SEMI_VALUE) !== 0;

    let node: any;

    switch (type) {
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
        const rawSel = css.slice(f1s, f1e);
        if (flags & FLAG_HAS_CLEAN_STR) {
          const cleanSel = buf.toString('utf8', strTableBase + clnS, strTableBase + clnE);
          node.selector = cleanSel;
          raws.selector = { value: cleanSel, raw: rawSel };
        } else {
          node.selector = rawSel;
        }
        break;
      }
      case 2: {
        node = Object.create(DeclProto);
        node.raws = {}; node[isClean] = false; node[my] = true;
        node.type = 'decl';
        node.prop = css.slice(f1s, f1e);
        const rawVal = css.slice(f2s, view[o + 11]);
        if (flags & FLAG_HAS_CLEAN_STR) {
          const cleanVal = buf.toString('utf8', strTableBase + clnS, strTableBase + clnE);
          node.value = cleanVal;
          raws.value = { value: cleanVal, raw: rawVal };
        } else {
          node.value = rawVal;
        }
        if (flags & FLAG_IMPORTANT) node.important = true;
        if (ex2S !== ABSENT32) raws.important = css.slice(ex2S, ex2E);
        break;
      }
      case 3: {
        node = Object.create(AtRuleProto);
        node.raws = {}; node[isClean] = false; node[my] = true;
        node.type = 'atrule';
        node.name   = css.slice(f1s, f1e);
        node.params = f2s !== ABSENT32 ? css.slice(f2s, view[o + 11]) : '';
        if (flags & FLAG_HAS_NODES) node.nodes = [];
        if (ex1S !== ABSENT32) raws.afterName = css.slice(ex1S, ex1E);
        break;
      }
      case 4: {
        node = Object.create(CommentProto);
        node.raws = {}; node[isClean] = false; node[my] = true;
        node.type = 'comment';
        node.text = css.slice(f1s, f1e);
        if (ex1S !== ABSENT32) raws.left  = css.slice(ex1S, ex1E);
        if (ex2S !== ABSENT32) raws.right = css.slice(ex2S, ex2E);
        break;
      }
      default:
        throw new Error(`Unknown offset node type: ${type}`);
    }

    node.raws   = raws;
    node.source = source;

    if (parentIdx !== ABSENT32) {
      const parent = nodes[parentIdx];
      if (parent.nodes === undefined) parent.nodes = [];
      parent.nodes.push(node);
      node.parent = parent;
    }

    nodes[i] = node;
  }

  return nodes[0] as Root;
}

// ─── Raw (no tree-walk) variants for benchmarking ─────────────────────────────

/** Rust parse → JSON string only (no JSON.parse, no buildNode) */
export function parseRustOnlySync(css: string, opts: ParseOptions = {}): string {
  return bindings.postcss.parseSync(css, opts.from ?? null);
}

/** Rust parse → JSON.parse only (no buildNode) */
export function parseJsonOnlySync(css: string, opts: ParseOptions = {}): any {
  return JSON.parse(bindings.postcss.parseSync(css, opts.from ?? null));
}

/** Rust parse → offset-array Buffer only (no JS tree walk) */
export function parseOffsetsRustOnlySync(css: string, opts: ParseOptions = {}): Buffer {
  return bindings.postcss.parseOffsetsSync(css, opts.from ?? null);
}

// ─── Public API ───────────────────────────────────────────────────────────────

export interface ParseOptions {
  from?: string;
  map?: any;
}

/**
 * Parse CSS into a PostCSS AST (async, JSON path).
 */
export async function parse(css: string, opts: ParseOptions = {}): Promise<Root> {
  const jsonStr = await bindings.postcss.parse(css, opts.from ?? null);
  const rustObj = JSON.parse(jsonStr);
  const input   = new postcss.Input(css, { from: opts.from });
  return buildNode(rustObj, input, undefined) as Root;
}

/**
 * Synchronous JSON parse (blocks the event loop).
 */
export function parseSync(css: string, opts: ParseOptions = {}): Root {
  const jsonStr = bindings.postcss.parseSync(css, opts.from ?? null);
  const rustObj = JSON.parse(jsonStr);
  const input   = new postcss.Input(css, { from: opts.from });
  return buildNode(rustObj, input, undefined) as Root;
}

/**
 * Synchronous offset-array parse — fastest path for large CSS.
 *
 * Rust emits a flat Uint32Array of CSS byte offsets; JS recovers every
 * string field with css.slice(start, end) in a single linear pass.
 */
export function parseOffsetsSync(css: string, opts: ParseOptions = {}): Root {
  const buf   = bindings.postcss.parseOffsetsSync(css, opts.from ?? null);
  const input = new postcss.Input(css, { from: opts.from });
  return buildNodeFromOffsets(buf, css, input);
}

/**
 * Create a PostCSS processor with Rust-powered parsing.
 */
export function postcssRs(plugins: Plugin[] = []): Processor {
  const processor = postcss.default(plugins);
  const originalProcess = processor.process.bind(processor);

  processor.process = function(css: string | Root, opts: ProcessOptions = {}): any {
    if (typeof css === 'string') {
      opts.parser = {
        parse: (cssStr: string | { toString(): string }, parseOpts?: any) => {
          const cssString = typeof cssStr === 'string' ? cssStr : cssStr.toString();
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

export const PostCSSRoot        = postcss.Root;
export const PostCSSRule        = postcss.Rule;
export const PostCSSAtRule      = postcss.AtRule;
export const PostCSSDeclaration = postcss.Declaration;
export const PostCSSComment     = postcss.Comment;
export const PostCSSContainer   = postcss.Container;
export const PostCSSNode        = postcss.Node;

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
