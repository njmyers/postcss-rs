/**
 * JSON parse path: Rust parse → JSON string → JSON.parse → buildNode.
 */

import * as path from 'node:path';
import * as postcss from 'postcss';
import type { Root } from 'postcss';

const bindings = require(path.join(__dirname, '../../index.js')) as {
  postcss: {
    parseToJsonAsync: (css: string, from?: null | string) => Promise<string>;
    parseToJsonSync: (css: string, from?: null | string) => string;
  };
};

const RootProto = postcss.Root.prototype;
const RuleProto = postcss.Rule.prototype;
const AtRuleProto = postcss.AtRule.prototype;
const DeclProto = postcss.Declaration.prototype;
const CommentProto = postcss.Comment.prototype;

const { isClean, my } = require('postcss/lib/symbols') as {
  isClean: symbol;
  my: symbol;
};

function buildNode(json: any, input: postcss.Input, parent: any): any {
  let node: any;

  switch (json.type) {
    case 'atrule': {
      node = Object.create(AtRuleProto);
      node.raws = {};
      node[isClean] = false;
      node[my] = true;
      node.type = 'atrule';
      node.name = json.name;
      node.params = json.params;
      break;
    }
    case 'comment': {
      node = Object.create(CommentProto);
      node.raws = {};
      node[isClean] = false;
      node[my] = true;
      node.type = 'comment';
      node.text = json.text;
      break;
    }
    case 'decl': {
      node = Object.create(DeclProto);
      node.raws = {};
      node[isClean] = false;
      node[my] = true;
      node.type = 'decl';
      node.prop = json.prop;
      node.value = json.value;
      if (json.important) {
        node.important = json.important;
      }
      break;
    }
    case 'root': {
      node = Object.create(RootProto);
      node.raws = {};
      node[isClean] = false;
      node[my] = true;
      node.type = 'root';
      node.nodes = [];
      break;
    }
    case 'rule': {
      node = Object.create(RuleProto);
      node.raws = {};
      node[isClean] = false;
      node[my] = true;
      node.type = 'rule';
      node.nodes = [];
      node.selector = json.selector;
      break;
    }
    default:
      throw new Error(`Unknown node type: ${json.type}`);
  }

  node.raws = json.raws;

  let s = json.source;
  node.source = s.end
    ? { end: s.end, input, start: s.start }
    : { input, start: s.start };

  if (parent !== undefined) {
    node.parent = parent;
  }

  let rawChildren = json.nodes;
  if (rawChildren) {
    let n = rawChildren.length;
    let built = new Array(n);
    for (let i = 0; i < n; i++) {
      built[i] = buildNode(rawChildren[i], input, node);
    }
    node.nodes = built;
  }

  return node;
}

export interface ParseOptions {
  from?: string;
}

/** Rust parse → JSON string only (no JSON.parse, no buildNode; for benchmarking) */
export function parseRustOnlySync(
  css: string,
  opts: ParseOptions = {}
): string {
  return bindings.postcss.parseToJsonSync(css, opts.from ?? null);
}

/** Rust parse → JSON.parse only (no buildNode; for benchmarking) */
export function parseJsonOnlySync(css: string, opts: ParseOptions = {}): any {
  return JSON.parse(bindings.postcss.parseToJsonSync(css, opts.from ?? null));
}

/** Parse CSS into a PostCSS AST via JSON (async). */
export async function parseJsonAsync(
  css: string,
  opts: ParseOptions = {}
): Promise<Root> {
  let jsonStr = await bindings.postcss.parseToJsonAsync(css, opts.from ?? null);
  let input = new postcss.Input(css, { from: opts.from });
  return buildNode(JSON.parse(jsonStr), input, undefined) as Root;
}

/** Parse CSS into a PostCSS AST via JSON (sync, blocks the event loop). */
export function parseJsonSync(css: string, opts: ParseOptions = {}): Root {
  let jsonStr = bindings.postcss.parseToJsonSync(css, opts.from ?? null);
  let input = new postcss.Input(css, { from: opts.from });
  return buildNode(JSON.parse(jsonStr), input, undefined) as Root;
}
