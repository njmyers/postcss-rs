/**
 * Offset-array parse path.
 *
 * Rust emits a flat Uint32Array of CSS byte offsets plus a UTF-8 string table.
 * JS recovers every string field with css.slice(start, end) — no JSON parsing,
 * no extra heap copies beyond what V8 already interns.
 *
 * Buffer layout:
 *   [u32 node_count] [node_count × STRIDE u32s] [UTF-8 string table]
 */

import * as postcss from 'postcss';
import type { Root } from 'postcss';
import * as path from 'path';
import type { NodeSlotLayout } from '../../index';

const bindings = require(path.join(__dirname, '../../index.js')) as {
  postcss: {
    parseToOffsetsSync: (css: string, from?: string | null) => Buffer;
    parseToOffsetsAsync: (css: string, from?: string | null) => Promise<Buffer>;
    nodeSlotLayout: () => NodeSlotLayout;
  };
};

// Fetch layout once at module startup.  Field indices are derived from the
// Rust NodeSlot struct via offset_of!, so they stay in sync automatically.
const L: NodeSlotLayout = bindings.postcss.nodeSlotLayout();

// Destructure into locals for minimal property-lookup overhead in the hot loop.
const STRIDE = L.stride;
const ABSENT32 = L.absent;

const IDX_TYPE_TAG = L.idxTypeTag;
const IDX_PARENT_IDX = L.idxParentIdx;
const IDX_SOURCE_START_OFF = L.idxSourceStartOff;
const IDX_SOURCE_START_LINE = L.idxSourceStartLine;
const IDX_SOURCE_START_COL = L.idxSourceStartCol;
const IDX_SOURCE_END_OFF = L.idxSourceEndOff;
const IDX_SOURCE_END_LINE = L.idxSourceEndLine;
const IDX_SOURCE_END_COL = L.idxSourceEndCol;
const IDX_FIELD1_START = L.idxField1Start;
const IDX_FIELD1_END = L.idxField1End;
const IDX_FIELD2_START = L.idxField2Start;
const IDX_FIELD2_END = L.idxField2End;
const IDX_RAWS_BEFORE_START = L.idxRawsBeforeStart;
const IDX_RAWS_BEFORE_END = L.idxRawsBeforeEnd;
const IDX_RAWS_BETWEEN_START = L.idxRawsBetweenStart;
const IDX_RAWS_BETWEEN_END = L.idxRawsBetweenEnd;
const IDX_RAWS_AFTER_START = L.idxRawsAfterStart;
const IDX_RAWS_AFTER_END = L.idxRawsAfterEnd;
const IDX_RAWS_EXTRA1_START = L.idxRawsExtra1Start;
const IDX_RAWS_EXTRA1_END = L.idxRawsExtra1End;
const IDX_RAWS_EXTRA2_START = L.idxRawsExtra2Start;
const IDX_RAWS_EXTRA2_END = L.idxRawsExtra2End;
const IDX_FLAGS = L.idxFlags;
const IDX_CLEAN_START = L.idxCleanStart;
const IDX_CLEAN_END = L.idxCleanEnd;

const FLAG_IMPORTANT = L.flagImportant;
const FLAG_SEMI_PRESENT = L.flagSemiPresent;
const FLAG_SEMI_VALUE = L.flagSemiValue;
const FLAG_HAS_CLEAN_STR = L.flagHasCleanStr;
const FLAG_HAS_NODES = L.flagHasNodes;

const TYPE_ROOT = L.typeRoot;
const TYPE_RULE = L.typeRule;
const TYPE_DECL = L.typeDecl;
const TYPE_ATRULE = L.typeAtrule;
const TYPE_COMMENT = L.typeComment;

const RootProto = postcss.Root.prototype;
const RuleProto = postcss.Rule.prototype;
const AtRuleProto = postcss.AtRule.prototype;
const DeclProto = postcss.Declaration.prototype;
const CommentProto = postcss.Comment.prototype;

const { isClean, my } = require('postcss/lib/symbols') as {
  isClean: symbol;
  my: symbol;
};

class OffsetParser {
  private static parseRoot(raws: any): any {
    const node = Object.create(RootProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'root';
    node.nodes = [];
    node.raws = raws;
    return node;
  }

  private static parseRule(
    buf: Buffer,
    css: string,
    strTableBase: number,
    f1s: number,
    f1e: number,
    ex1S: number,
    ex1E: number,
    flags: number,
    clnS: number,
    clnE: number,
    raws: any
  ): any {
    const node = Object.create(RuleProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'rule';
    node.nodes = [];
    const rawSel = css.slice(f1s, f1e);
    if (flags & FLAG_HAS_CLEAN_STR) {
      const cleanSel = buf.toString(
        'utf8',
        strTableBase + clnS,
        strTableBase + clnE
      );
      node.selector = cleanSel;
      raws.selector = { value: cleanSel, raw: rawSel };
    } else {
      node.selector = rawSel;
    }
    // ex1S/ex1E hold ownSemicolon for RULE (unused slot repurposed)
    if (ex1S !== ABSENT32) {
      raws.ownSemicolon = css.slice(ex1S, ex1E);
    }
    node.raws = raws;
    return node;
  }

  private static parseDeclaration(
    buf: Buffer,
    css: string,
    strTableBase: number,
    f1s: number,
    f1e: number,
    f2s: number,
    f2e: number,
    ex2S: number,
    ex2E: number,
    flags: number,
    clnS: number,
    clnE: number,
    raws: any
  ): any {
    const node = Object.create(DeclProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'decl';
    node.prop = css.slice(f1s, f1e);
    const rawVal = css.slice(f2s, f2e);
    if (flags & FLAG_HAS_CLEAN_STR) {
      const cleanVal = buf.toString(
        'utf8',
        strTableBase + clnS,
        strTableBase + clnE
      );
      node.value = cleanVal;
      raws.value = { value: cleanVal, raw: rawVal };
    } else {
      node.value = rawVal;
    }
    if (flags & FLAG_IMPORTANT) {
      node.important = true;
    }
    if (ex2S !== ABSENT32) {
      raws.important = css.slice(ex2S, ex2E);
    }
    node.raws = raws;
    return node;
  }

  private static parseAtRule(
    css: string,
    f1s: number,
    f1e: number,
    f2s: number,
    f2e: number,
    ex1S: number,
    ex1E: number,
    flags: number,
    raws: any
  ): any {
    const node = Object.create(AtRuleProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'atrule';
    node.name = css.slice(f1s, f1e);
    node.params = f2s !== ABSENT32 ? css.slice(f2s, f2e) : '';
    if (flags & FLAG_HAS_NODES) {
      node.nodes = [];
    }
    if (ex1S !== ABSENT32) {
      raws.afterName = css.slice(ex1S, ex1E);
    }
    node.raws = raws;
    return node;
  }

  private static parseComment(
    css: string,
    f1s: number,
    f1e: number,
    ex1S: number,
    ex1E: number,
    ex2S: number,
    ex2E: number,
    raws: any
  ): any {
    const node = Object.create(CommentProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'comment';
    node.text = css.slice(f1s, f1e);
    if (ex1S !== ABSENT32) {
      raws.left = css.slice(ex1S, ex1E);
    }
    if (ex2S !== ABSENT32) {
      raws.right = css.slice(ex2S, ex2E);
    }
    node.raws = raws;
    return node;
  }

  static parse(buf: Buffer, css: string, input: postcss.Input): Root {
    // Align-safe Uint32Array view (host is LE on all Node.js platforms we target)
    let view: Uint32Array;
    if (buf.byteOffset % 4 === 0) {
      view = new Uint32Array(buf.buffer, buf.byteOffset, buf.byteLength >>> 2);
    } else {
      const copy = new Uint8Array(buf.byteLength);
      copy.set(new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength));
      view = new Uint32Array(copy.buffer);
    }

    const nodeCount = view[0];
    const strTableBase = (1 + nodeCount * STRIDE) * 4;
    const nodes: any[] = new Array(nodeCount);

    for (let i = 0; i < nodeCount; i++) {
      const o = 1 + i * STRIDE;

      const type = view[o + IDX_TYPE_TAG];
      const parentIdx = view[o + IDX_PARENT_IDX];
      const ssOff = view[o + IDX_SOURCE_START_OFF];
      const ssLine = view[o + IDX_SOURCE_START_LINE];
      const ssCol = view[o + IDX_SOURCE_START_COL];
      const seOff = view[o + IDX_SOURCE_END_OFF];
      const seLine = view[o + IDX_SOURCE_END_LINE];
      const seCol = view[o + IDX_SOURCE_END_COL];
      const f1s = view[o + IDX_FIELD1_START];
      const f1e = view[o + IDX_FIELD1_END];
      const f2s = view[o + IDX_FIELD2_START];
      const f2e = view[o + IDX_FIELD2_END];
      const bfS = view[o + IDX_RAWS_BEFORE_START];
      const bfE = view[o + IDX_RAWS_BEFORE_END];
      const btS = view[o + IDX_RAWS_BETWEEN_START];
      const btE = view[o + IDX_RAWS_BETWEEN_END];
      const afS = view[o + IDX_RAWS_AFTER_START];
      const afE = view[o + IDX_RAWS_AFTER_END];
      const ex1S = view[o + IDX_RAWS_EXTRA1_START];
      const ex1E = view[o + IDX_RAWS_EXTRA1_END];
      const ex2S = view[o + IDX_RAWS_EXTRA2_START];
      const ex2E = view[o + IDX_RAWS_EXTRA2_END];
      const flags = view[o + IDX_FLAGS];
      const clnS = view[o + IDX_CLEAN_START];
      const clnE = view[o + IDX_CLEAN_END];

      const start = { offset: ssOff, line: ssLine, column: ssCol };
      const source: any =
        seOff !== ABSENT32
          ? {
              start,
              end: { offset: seOff, line: seLine, column: seCol },
              input,
            }
          : { start, input };

      const raws: any = {};
      if (bfS !== ABSENT32) {
        raws.before = css.slice(bfS, bfE);
      }
      if (btS !== ABSENT32) {
        raws.between = css.slice(btS, btE);
      }
      if (afS !== ABSENT32) {
        raws.after = css.slice(afS, afE);
      }
      if (flags & FLAG_SEMI_PRESENT) {
        raws.semicolon = (flags & FLAG_SEMI_VALUE) !== 0;
      }

      let node: any;

      switch (type) {
        case TYPE_ROOT:
          node = OffsetParser.parseRoot(raws);
          break;
        case TYPE_RULE:
          node = OffsetParser.parseRule(
            buf,
            css,
            strTableBase,
            f1s,
            f1e,
            ex1S,
            ex1E,
            flags,
            clnS,
            clnE,
            raws
          );
          break;
        case TYPE_DECL:
          node = OffsetParser.parseDeclaration(
            buf,
            css,
            strTableBase,
            f1s,
            f1e,
            f2s,
            f2e,
            ex2S,
            ex2E,
            flags,
            clnS,
            clnE,
            raws
          );
          break;
        case TYPE_ATRULE:
          node = OffsetParser.parseAtRule(
            css,
            f1s,
            f1e,
            f2s,
            f2e,
            ex1S,
            ex1E,
            flags,
            raws
          );
          break;
        case TYPE_COMMENT:
          node = OffsetParser.parseComment(
            css,
            f1s,
            f1e,
            ex1S,
            ex1E,
            ex2S,
            ex2E,
            raws
          );
          break;
        default:
          throw new Error(`Unknown offset node type: ${type}`);
      }

      node.source = source;

      if (parentIdx !== ABSENT32) {
        const parent = nodes[parentIdx];
        if (parent.nodes === undefined) {
          parent.nodes = [];
        }
        parent.nodes.push(node);
        node.parent = parent;
      }

      nodes[i] = node;
    }

    return nodes[0] as Root;
  }
}

export interface ParseOptions {
  from?: string;
}

/** Synchronous offset-array parse — fastest path for large CSS. */
export function parseOffsetsSync(css: string, opts: ParseOptions = {}): Root {
  const buf = bindings.postcss.parseToOffsetsSync(css, opts.from ?? null);
  const input = new postcss.Input(css, { from: opts.from });
  return OffsetParser.parse(buf, css, input);
}

/** Async offset-array parse — runs on the thread pool, non-blocking. */
export async function parseOffsetsAsync(
  css: string,
  opts: ParseOptions = {}
): Promise<Root> {
  const buf = await bindings.postcss.parseToOffsetsAsync(
    css,
    opts.from ?? null
  );
  const input = new postcss.Input(css, { from: opts.from });
  return OffsetParser.parse(buf, css, input);
}

/** Rust parse → offset-array Buffer only (no JS tree walk; for benchmarking) */
export function parseOffsetsRustOnlySync(
  css: string,
  opts: ParseOptions = {}
): Buffer {
  return bindings.postcss.parseToOffsetsSync(css, opts.from ?? null);
}
