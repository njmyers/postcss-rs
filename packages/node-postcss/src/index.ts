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
 * The offset path is fastest for large CSS (>10 KB).
 */

import * as postcss from 'postcss';
import type { Root, Plugin, Processor, ProcessOptions } from 'postcss';

export type { ParseOptions } from './parsers/json';
export {
  parseJsonAsync,
  parseJsonSync,
  parseRustOnlySync,
  parseJsonOnlySync,
} from './parsers/json';
export {
  parseOffsetsSync,
  parseOffsetsAsync,
  parseOffsetsRustOnlySync,
} from './parsers/offset';

import { parseOffsetsSync } from './parsers/offset';

/** PostCSS Processor subclass with Rust-powered parsing and postcss stringify. */
class RustProcessor extends postcss.Processor {
  constructor(plugins: Plugin[] = []) {
    super(plugins);
  }

  process(css: string | Root, opts: ProcessOptions = {}): any {
    const mergedOpts: ProcessOptions = { ...opts };
    if (typeof css === 'string') {
      if (!mergedOpts.parser) {
        mergedOpts.parser = {
          parse: (cssStr: string | { toString(): string }, parseOpts?: any) => {
            const cssString = typeof cssStr === 'string' ? cssStr : cssStr.toString();
            return parseOffsetsSync(cssString, parseOpts || {}) as any;
          },
        };
      }
      if (!mergedOpts.stringifier) {
        mergedOpts.stringifier = postcss.stringify;
      }
    }
    return super.process(css, mergedOpts);
  }
}

/**
 * Create a PostCSS processor with Rust-powered parsing.
 */
export function postcssRs(plugins: Plugin[] = []): Processor {
  return new RustProcessor(plugins);
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
