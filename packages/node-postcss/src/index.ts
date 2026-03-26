/**
 * @postcss-rs/node-postcss
 *
 * A high-performance PostCSS-compatible parser powered by Rust.
 *
 * Two parse paths are available:
 *
 *   parseSync / parse      — Rust parse → JSON → JSON.parse → buildNode
 *   parseOffsetsSync       — Rust parse → flat Uint32Array of byte offsets →
 *                            buildNodeFromOffsets (css.slice for all strings)
 *
 * The offset path is the default and fastest for large CSS (>10 KB).
 */

import * as postcss from 'postcss';
import type { Root, Plugin, Processor, ProcessOptions } from 'postcss';
import * as path from 'path';
import { parseOffsetsSync, parseOffsetsRustOnlySync } from './offset';

export type { ParseOptions } from './offset';
export { parseOffsetsSync, parseOffsetsRustOnlySync };

// ─── Bindings ─────────────────────────────────────────────────────────────────

// Load Rust native bindings via the napi-rs v3 generated loader.
// __dirname is dist/, index.js is in the package root (one level up).
const bindings = require(path.join(__dirname, '../index.js')) as {
  postcss: {
    parse:     (css: string, from?: string | null) => Promise<string>;
    parseSync: (css: string, from?: string | null) => string;
    stringify: (json: string) => string;
  };
};

// ─── PostCSS internals ────────────────────────────────────────────────────────

// Cache class constructors at module level to avoid repeated property lookups
// inside the hot buildNode loop.
const RootProto    = postcss.Root.prototype;
const RuleProto    = postcss.Rule.prototype;
const AtRuleProto  = postcss.AtRule.prototype;
const DeclProto    = postcss.Declaration.prototype;
const CommentProto = postcss.Comment.prototype;

// PostCSS internal symbols used by the Node constructor.  Set manually when
// bypassing the constructor via Object.create() so PostCSS's internal guards
// (e.g. `!i[my]` in normalize()) pass correctly.
const { isClean, my } = require('postcss/lib/symbols') as { isClean: symbol; my: symbol };

// ─── JSON-path builder ────────────────────────────────────────────────────────

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

// ─── Raw (no tree-walk) variants for benchmarking ─────────────────────────────

/** Rust parse → JSON string only (no JSON.parse, no buildNode) */
export function parseRustOnlySync(css: string, opts: { from?: string } = {}): string {
  return bindings.postcss.parseSync(css, opts.from ?? null);
}

/** Rust parse → JSON.parse only (no buildNode) */
export function parseJsonOnlySync(css: string, opts: { from?: string } = {}): any {
  return JSON.parse(bindings.postcss.parseSync(css, opts.from ?? null));
}

// ─── Public API ───────────────────────────────────────────────────────────────

/**
 * Parse CSS into a PostCSS AST (async, JSON path).
 */
export async function parse(css: string, opts: { from?: string } = {}): Promise<Root> {
  const jsonStr = await bindings.postcss.parse(css, opts.from ?? null);
  const rustObj = JSON.parse(jsonStr);
  const input   = new postcss.Input(css, { from: opts.from });
  return buildNode(rustObj, input, undefined) as Root;
}

/**
 * Synchronous JSON parse (blocks the event loop).
 */
export function parseSync(css: string, opts: { from?: string } = {}): Root {
  const jsonStr = bindings.postcss.parseSync(css, opts.from ?? null);
  const rustObj = JSON.parse(jsonStr);
  const input   = new postcss.Input(css, { from: opts.from });
  return buildNode(rustObj, input, undefined) as Root;
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
