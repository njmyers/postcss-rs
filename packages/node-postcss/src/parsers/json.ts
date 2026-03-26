/**
 * JSON parse path: Rust parse → JSON string → JSON.parse → buildNode.
 */

import * as postcss from 'postcss';
import type { Root } from 'postcss';
import * as path from 'path';

// ─── Bindings ─────────────────────────────────────────────────────────────────

const bindings = require(path.join(__dirname, '../../index.js')) as {
  postcss: {
    parse:     (css: string, from?: string | null) => Promise<string>;
    parseSync: (css: string, from?: string | null) => string;
  };
};

// ─── PostCSS internals ────────────────────────────────────────────────────────

const RootProto    = postcss.Root.prototype;
const RuleProto    = postcss.Rule.prototype;
const AtRuleProto  = postcss.AtRule.prototype;
const DeclProto    = postcss.Declaration.prototype;
const CommentProto = postcss.Comment.prototype;

const { isClean, my } = require('postcss/lib/symbols') as { isClean: symbol; my: symbol };

// ─── Builder ──────────────────────────────────────────────────────────────────

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

// ─── Public API ───────────────────────────────────────────────────────────────

export interface ParseOptions {
  from?: string;
}

/** Rust parse → JSON string only (no JSON.parse, no buildNode; for benchmarking) */
export function parseRustOnlySync(css: string, opts: ParseOptions = {}): string {
  return bindings.postcss.parseSync(css, opts.from ?? null);
}

/** Rust parse → JSON.parse only (no buildNode; for benchmarking) */
export function parseJsonOnlySync(css: string, opts: ParseOptions = {}): any {
  return JSON.parse(bindings.postcss.parseSync(css, opts.from ?? null));
}

/** Parse CSS into a PostCSS AST (async). */
export async function parse(css: string, opts: ParseOptions = {}): Promise<Root> {
  const jsonStr = await bindings.postcss.parse(css, opts.from ?? null);
  const input   = new postcss.Input(css, { from: opts.from });
  return buildNode(JSON.parse(jsonStr), input, undefined) as Root;
}

/** Synchronous JSON parse (blocks the event loop). */
export function parseSync(css: string, opts: ParseOptions = {}): Root {
  const jsonStr = bindings.postcss.parseSync(css, opts.from ?? null);
  const input   = new postcss.Input(css, { from: opts.from });
  return buildNode(JSON.parse(jsonStr), input, undefined) as Root;
}
