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
const layout: NodeSlotLayout = bindings.postcss.nodeSlotLayout();

// Destructure into locals for minimal property-lookup overhead in the hot loop.
const STRIDE = layout.stride;
const ABSENT = layout.absent;

const IDX_TYPE_TAG = layout.idxTypeTag;
const IDX_PARENT_IDX = layout.idxParentIdx;
const IDX_SOURCE_START_OFFSET = layout.idxSourceStartOff;
const IDX_SOURCE_START_LINE = layout.idxSourceStartLine;
const IDX_SOURCE_START_COLUMN = layout.idxSourceStartCol;
const IDX_SOURCE_END_OFFSET = layout.idxSourceEndOff;
const IDX_SOURCE_END_LINE = layout.idxSourceEndLine;
const IDX_SOURCE_END_COLUMN = layout.idxSourceEndCol;
const IDX_FIELD1_START = layout.idxField1Start;
const IDX_FIELD1_END = layout.idxField1End;
const IDX_FIELD2_START = layout.idxField2Start;
const IDX_FIELD2_END = layout.idxField2End;
const IDX_RAWS_BEFORE_START = layout.idxRawsBeforeStart;
const IDX_RAWS_BEFORE_END = layout.idxRawsBeforeEnd;
const IDX_RAWS_BETWEEN_START = layout.idxRawsBetweenStart;
const IDX_RAWS_BETWEEN_END = layout.idxRawsBetweenEnd;
const IDX_RAWS_AFTER_START = layout.idxRawsAfterStart;
const IDX_RAWS_AFTER_END = layout.idxRawsAfterEnd;
const IDX_RAWS_EXTRA1_START = layout.idxRawsExtra1Start;
const IDX_RAWS_EXTRA1_END = layout.idxRawsExtra1End;
const IDX_RAWS_EXTRA2_START = layout.idxRawsExtra2Start;
const IDX_RAWS_EXTRA2_END = layout.idxRawsExtra2End;
const IDX_FLAGS = layout.idxFlags;
const IDX_CLEAN_START = layout.idxCleanStart;
const IDX_CLEAN_END = layout.idxCleanEnd;

const FLAG_IMPORTANT = layout.flagImportant;
const FLAG_SEMI_PRESENT = layout.flagSemiPresent;
const FLAG_SEMI_VALUE = layout.flagSemiValue;
const FLAG_HAS_CLEAN_STRING = layout.flagHasCleanStr;
const FLAG_HAS_NODES = layout.flagHasNodes;

const TYPE_ROOT = layout.typeRoot;
const TYPE_RULE = layout.typeRule;
const TYPE_DECL = layout.typeDecl;
const TYPE_ATRULE = layout.typeAtrule;
const TYPE_COMMENT = layout.typeComment;

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
    stringTableBase: number,
    field1Start: number,
    field1End: number,
    rawsExtra1Start: number,
    rawsExtra1End: number,
    flags: number,
    cleanStart: number,
    cleanEnd: number,
    raws: any
  ): any {
    const node = Object.create(RuleProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'rule';
    node.nodes = [];
    const rawSelector = css.slice(field1Start, field1End);
    if (flags & FLAG_HAS_CLEAN_STRING) {
      const cleanSelector = buf.toString(
        'utf8',
        stringTableBase + cleanStart,
        stringTableBase + cleanEnd
      );
      node.selector = cleanSelector;
      raws.selector = { value: cleanSelector, raw: rawSelector };
    } else {
      node.selector = rawSelector;
    }
    // rawsExtra1 holds ownSemicolon for RULE (unused slot repurposed)
    if (rawsExtra1Start !== ABSENT) {
      raws.ownSemicolon = css.slice(rawsExtra1Start, rawsExtra1End);
    }
    node.raws = raws;
    return node;
  }

  private static parseDeclaration(
    buf: Buffer,
    css: string,
    stringTableBase: number,
    field1Start: number,
    field1End: number,
    field2Start: number,
    field2End: number,
    rawsExtra2Start: number,
    rawsExtra2End: number,
    flags: number,
    cleanStart: number,
    cleanEnd: number,
    raws: any
  ): any {
    const node = Object.create(DeclProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'decl';
    node.prop = css.slice(field1Start, field1End);
    const rawValue = css.slice(field2Start, field2End);
    if (flags & FLAG_HAS_CLEAN_STRING) {
      const cleanValue = buf.toString(
        'utf8',
        stringTableBase + cleanStart,
        stringTableBase + cleanEnd
      );
      node.value = cleanValue;
      raws.value = { value: cleanValue, raw: rawValue };
    } else {
      node.value = rawValue;
    }
    if (flags & FLAG_IMPORTANT) {
      node.important = true;
    }
    if (rawsExtra2Start !== ABSENT) {
      raws.important = css.slice(rawsExtra2Start, rawsExtra2End);
    }
    node.raws = raws;
    return node;
  }

  private static parseAtRule(
    css: string,
    field1Start: number,
    field1End: number,
    field2Start: number,
    field2End: number,
    rawsExtra1Start: number,
    rawsExtra1End: number,
    flags: number,
    raws: any
  ): any {
    const node = Object.create(AtRuleProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'atrule';
    node.name = css.slice(field1Start, field1End);
    node.params =
      field2Start !== ABSENT ? css.slice(field2Start, field2End) : '';
    if (flags & FLAG_HAS_NODES) {
      node.nodes = [];
    }
    if (rawsExtra1Start !== ABSENT) {
      raws.afterName = css.slice(rawsExtra1Start, rawsExtra1End);
    }
    node.raws = raws;
    return node;
  }

  private static parseComment(
    css: string,
    field1Start: number,
    field1End: number,
    rawsExtra1Start: number,
    rawsExtra1End: number,
    rawsExtra2Start: number,
    rawsExtra2End: number,
    raws: any
  ): any {
    const node = Object.create(CommentProto);
    node[isClean] = false;
    node[my] = true;
    node.type = 'comment';
    node.text = css.slice(field1Start, field1End);
    if (rawsExtra1Start !== ABSENT) {
      raws.left = css.slice(rawsExtra1Start, rawsExtra1End);
    }
    if (rawsExtra2Start !== ABSENT) {
      raws.right = css.slice(rawsExtra2Start, rawsExtra2End);
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
    const stringTableBase = (1 + nodeCount * STRIDE) * 4;
    const nodes: any[] = new Array(nodeCount);

    for (let i = 0; i < nodeCount; i++) {
      const slotOffset = 1 + i * STRIDE;

      const type = view[slotOffset + IDX_TYPE_TAG];
      const parentIdx = view[slotOffset + IDX_PARENT_IDX];
      const sourceStartOffset = view[slotOffset + IDX_SOURCE_START_OFFSET];
      const sourceStartLine = view[slotOffset + IDX_SOURCE_START_LINE];
      const sourceStartColumn = view[slotOffset + IDX_SOURCE_START_COLUMN];
      const sourceEndOffset = view[slotOffset + IDX_SOURCE_END_OFFSET];
      const sourceEndLine = view[slotOffset + IDX_SOURCE_END_LINE];
      const sourceEndColumn = view[slotOffset + IDX_SOURCE_END_COLUMN];
      const field1Start = view[slotOffset + IDX_FIELD1_START];
      const field1End = view[slotOffset + IDX_FIELD1_END];
      const field2Start = view[slotOffset + IDX_FIELD2_START];
      const field2End = view[slotOffset + IDX_FIELD2_END];
      const rawsBeforeStart = view[slotOffset + IDX_RAWS_BEFORE_START];
      const rawsBeforeEnd = view[slotOffset + IDX_RAWS_BEFORE_END];
      const rawsBetweenStart = view[slotOffset + IDX_RAWS_BETWEEN_START];
      const rawsBetweenEnd = view[slotOffset + IDX_RAWS_BETWEEN_END];
      const rawsAfterStart = view[slotOffset + IDX_RAWS_AFTER_START];
      const rawsAfterEnd = view[slotOffset + IDX_RAWS_AFTER_END];
      const rawsExtra1Start = view[slotOffset + IDX_RAWS_EXTRA1_START];
      const rawsExtra1End = view[slotOffset + IDX_RAWS_EXTRA1_END];
      const rawsExtra2Start = view[slotOffset + IDX_RAWS_EXTRA2_START];
      const rawsExtra2End = view[slotOffset + IDX_RAWS_EXTRA2_END];
      const flags = view[slotOffset + IDX_FLAGS];
      const cleanStart = view[slotOffset + IDX_CLEAN_START];
      const cleanEnd = view[slotOffset + IDX_CLEAN_END];

      const start = {
        offset: sourceStartOffset,
        line: sourceStartLine,
        column: sourceStartColumn,
      };
      const source: any =
        sourceEndOffset !== ABSENT
          ? {
              start,
              end: {
                offset: sourceEndOffset,
                line: sourceEndLine,
                column: sourceEndColumn,
              },
              input,
            }
          : { start, input };

      const raws: any = {};
      if (rawsBeforeStart !== ABSENT) {
        raws.before = css.slice(rawsBeforeStart, rawsBeforeEnd);
      }
      if (rawsBetweenStart !== ABSENT) {
        raws.between = css.slice(rawsBetweenStart, rawsBetweenEnd);
      }
      if (rawsAfterStart !== ABSENT) {
        raws.after = css.slice(rawsAfterStart, rawsAfterEnd);
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
            stringTableBase,
            field1Start,
            field1End,
            rawsExtra1Start,
            rawsExtra1End,
            flags,
            cleanStart,
            cleanEnd,
            raws
          );
          break;
        case TYPE_DECL:
          node = OffsetParser.parseDeclaration(
            buf,
            css,
            stringTableBase,
            field1Start,
            field1End,
            field2Start,
            field2End,
            rawsExtra2Start,
            rawsExtra2End,
            flags,
            cleanStart,
            cleanEnd,
            raws
          );
          break;
        case TYPE_ATRULE:
          node = OffsetParser.parseAtRule(
            css,
            field1Start,
            field1End,
            field2Start,
            field2End,
            rawsExtra1Start,
            rawsExtra1End,
            flags,
            raws
          );
          break;
        case TYPE_COMMENT:
          node = OffsetParser.parseComment(
            css,
            field1Start,
            field1End,
            rawsExtra1Start,
            rawsExtra1End,
            rawsExtra2Start,
            rawsExtra2End,
            raws
          );
          break;
        default:
          throw new Error(`Unknown offset node type: ${type}`);
      }

      node.source = source;

      if (parentIdx !== ABSENT) {
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
