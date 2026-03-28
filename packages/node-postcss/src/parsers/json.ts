/**
 * JSON parse path: Rust parse → JSON string → JSON.parse → buildNode.
 */

import * as postcss from 'postcss';
import type { Root } from 'postcss';
import * as path from 'path';

const bindings = require(path.join(__dirname, '../../index.js')) as {
  postcss: {
    parseToJsonAsync: (css: string, from?: string | null) => Promise<string>;
    parseToJsonSync: (css: string, from?: string | null) => string;
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
    case 'comment': {
      node = Object.create(CommentProto);
      node.raws = {};
      node[isClean] = false;
      node[my] = true;
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

  if (parent !== undefined) {
    node.parent = parent;
  }

  const rawChildren = json.nodes;
  if (rawChildren) {
    const n = rawChildren.length;
    const built = new Array(n);
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

/** Parse CSS into a PostCSS AST via JSON (async). */
export async function parseJsonAsync(
  css: string,
  opts: ParseOptions = {}
): Promise<Root> {
  const jsonStr = await bindings.postcss.parseToJsonAsync(
    css,
    opts.from ?? null
  );
  const input = new postcss.Input(css, { from: opts.from });
  return buildNode(JSON.parse(jsonStr), input, undefined) as Root;
}

/** Parse CSS into a PostCSS AST via JSON (sync, blocks the event loop). */
export function parseJsonSync(css: string, opts: ParseOptions = {}): Root {
  const jsonStr = bindings.postcss.parseToJsonSync(css, opts.from ?? null);
  const input = new postcss.Input(css, { from: opts.from });
  return buildNode(JSON.parse(jsonStr), input, undefined) as Root;
}
