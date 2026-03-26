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
  parse,
  parseSync,
  parseRustOnlySync,
  parseJsonOnlySync,
} from './parsers/json';
export {
  parseOffsetsSync,
  parseOffsetsAsync,
  parseOffsetsRustOnlySync,
} from './parsers/offset';

// Re-import parse for use in postcssRs below
import { parse } from './parsers/json';

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
