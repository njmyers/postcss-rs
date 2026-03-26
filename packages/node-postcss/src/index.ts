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
    // Layout descriptor — field indices derived from the Rust NodeSlot struct
    nodeSlotLayout: () => NodeSlotLayout;
  };
};

// ─── NodeSlotLayout ──────────────────────────────────────────────────────────
// Mirrors the Rust `NodeSlotLayout` napi object.  All idx_* fields are u32
// slot indices (not byte offsets) within a single node's STRIDE-wide block.

interface NodeSlotLayout {
  // Node type constants
  typeRoot: number; typeRule: number; typeDecl: number;
  typeAtrule: number; typeComment: number;
  // Layout
  stride: number; absent: number;
  // Slot indices
  idxTypeTag: number; idxParentIdx: number;
  idxSourceStartOff: number; idxSourceStartLine: number; idxSourceStartCol: number;
  idxSourceEndOff: number; idxSourceEndLine: number; idxSourceEndCol: number;
  idxField1Start: number; idxField1End: number;
  idxField2Start: number; idxField2End: number;
  idxRawsBeforeStart: number; idxRawsBeforeEnd: number;
  idxRawsBetweenStart: number; idxRawsBetweenEnd: number;
  idxRawsAfterStart: number; idxRawsAfterEnd: number;
  idxRawsExtra1Start: number; idxRawsExtra1End: number;
  idxRawsExtra2Start: number; idxRawsExtra2End: number;
  idxFlags: number; idxCleanStart: number; idxCleanEnd: number;
  // Flag bits
  flagImportant: number; flagSemiPresent: number; flagSemiValue: number;
  flagHasCleanStr: number; flagCleanIsField2: number; flagHasNodes: number;
}

// Fetch the layout once at module startup.  All field indices are derived from
// the Rust NodeSlot struct via offset_of!, so they stay in sync automatically.
const L: NodeSlotLayout = bindings.postcss.nodeSlotLayout();

// Destructure into local constants for minimal property-lookup overhead in the
// hot buildNodeFromOffsets loop.
const STRIDE   = L.stride;
const ABSENT32 = L.absent;

const IDX_TYPE_TAG           = L.idxTypeTag;
const IDX_PARENT_IDX         = L.idxParentIdx;
const IDX_SOURCE_START_OFF   = L.idxSourceStartOff;
const IDX_SOURCE_START_LINE  = L.idxSourceStartLine;
const IDX_SOURCE_START_COL   = L.idxSourceStartCol;
const IDX_SOURCE_END_OFF     = L.idxSourceEndOff;
const IDX_SOURCE_END_LINE    = L.idxSourceEndLine;
const IDX_SOURCE_END_COL     = L.idxSourceEndCol;
const IDX_FIELD1_START       = L.idxField1Start;
const IDX_FIELD1_END         = L.idxField1End;
const IDX_FIELD2_START       = L.idxField2Start;
const IDX_FIELD2_END         = L.idxField2End;
const IDX_RAWS_BEFORE_START  = L.idxRawsBeforeStart;
const IDX_RAWS_BEFORE_END    = L.idxRawsBeforeEnd;
const IDX_RAWS_BETWEEN_START = L.idxRawsBetweenStart;
const IDX_RAWS_BETWEEN_END   = L.idxRawsBetweenEnd;
const IDX_RAWS_AFTER_START   = L.idxRawsAfterStart;
const IDX_RAWS_AFTER_END     = L.idxRawsAfterEnd;
const IDX_RAWS_EXTRA1_START  = L.idxRawsExtra1Start;
const IDX_RAWS_EXTRA1_END    = L.idxRawsExtra1End;
const IDX_RAWS_EXTRA2_START  = L.idxRawsExtra2Start;
const IDX_RAWS_EXTRA2_END    = L.idxRawsExtra2End;
const IDX_FLAGS              = L.idxFlags;
const IDX_CLEAN_START        = L.idxCleanStart;
const IDX_CLEAN_END          = L.idxCleanEnd;

const FLAG_IMPORTANT       = L.flagImportant;
const FLAG_SEMI_PRESENT    = L.flagSemiPresent;
const FLAG_SEMI_VALUE      = L.flagSemiValue;
const FLAG_HAS_CLEAN_STR   = L.flagHasCleanStr;
const FLAG_HAS_NODES       = L.flagHasNodes;

const TYPE_ROOT    = L.typeRoot;
const TYPE_RULE    = L.typeRule;
const TYPE_DECL    = L.typeDecl;
const TYPE_ATRULE  = L.typeAtrule;
const TYPE_COMMENT = L.typeComment;

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

  const nodeCount    = view[0];
  const strTableBase = (1 + nodeCount * STRIDE) * 4;
  const nodes: any[] = new Array(nodeCount);

  for (let i = 0; i < nodeCount; i++) {
    const o = 1 + i * STRIDE;

    const type      = view[o + IDX_TYPE_TAG];
    const parentIdx = view[o + IDX_PARENT_IDX];
    const ssOff     = view[o + IDX_SOURCE_START_OFF];
    const ssLine    = view[o + IDX_SOURCE_START_LINE];
    const ssCol     = view[o + IDX_SOURCE_START_COL];
    const seOff     = view[o + IDX_SOURCE_END_OFF];
    const seLine    = view[o + IDX_SOURCE_END_LINE];
    const seCol     = view[o + IDX_SOURCE_END_COL];
    const f1s       = view[o + IDX_FIELD1_START];
    const f1e       = view[o + IDX_FIELD1_END];
    const f2s       = view[o + IDX_FIELD2_START];
    const f2e       = view[o + IDX_FIELD2_END];
    const bfS       = view[o + IDX_RAWS_BEFORE_START];
    const bfE       = view[o + IDX_RAWS_BEFORE_END];
    const btS       = view[o + IDX_RAWS_BETWEEN_START];
    const btE       = view[o + IDX_RAWS_BETWEEN_END];
    const afS       = view[o + IDX_RAWS_AFTER_START];
    const afE       = view[o + IDX_RAWS_AFTER_END];
    const ex1S      = view[o + IDX_RAWS_EXTRA1_START];
    const ex1E      = view[o + IDX_RAWS_EXTRA1_END];
    const ex2S      = view[o + IDX_RAWS_EXTRA2_START];
    const ex2E      = view[o + IDX_RAWS_EXTRA2_END];
    const flags     = view[o + IDX_FLAGS];
    const clnS      = view[o + IDX_CLEAN_START];
    const clnE      = view[o + IDX_CLEAN_END];

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
      case TYPE_ROOT: {
        node = Object.create(RootProto);
        node.raws = {}; node[isClean] = false; node[my] = true;
        node.type = 'root'; node.nodes = [];
        break;
      }
      case TYPE_RULE: {
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
      case TYPE_DECL: {
        node = Object.create(DeclProto);
        node.raws = {}; node[isClean] = false; node[my] = true;
        node.type = 'decl';
        node.prop = css.slice(f1s, f1e);
        const rawVal = css.slice(f2s, f2e);
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
      case TYPE_ATRULE: {
        node = Object.create(AtRuleProto);
        node.raws = {}; node[isClean] = false; node[my] = true;
        node.type = 'atrule';
        node.name   = css.slice(f1s, f1e);
        node.params = f2s !== ABSENT32 ? css.slice(f2s, f2e) : '';
        if (flags & FLAG_HAS_NODES) node.nodes = [];
        if (ex1S !== ABSENT32) raws.afterName = css.slice(ex1S, ex1E);
        break;
      }
      case TYPE_COMMENT: {
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
