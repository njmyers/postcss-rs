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
import type { Plugin, Processor } from 'postcss';

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
export { RustProcessorFactory } from './processor';
export type { ParseFn } from './processor';

import { RustProcessorFactory } from './processor';
import { parseOffsetsSync } from './parsers/offset';

/**
 * Create a PostCSS processor with Rust-powered parsing (offset path).
 */
const OffsetProcessor = RustProcessorFactory(parseOffsetsSync);

export function postcssRs(plugins: Plugin[] = []): Processor {
  return new OffsetProcessor(plugins);
}

// Default export
export default postcssRs;

// Re-export PostCSS for convenience
export { postcssRs as postcss };

export const PostCSSRoot = postcss.Root;
export const PostCSSRule = postcss.Rule;
export const PostCSSAtRule = postcss.AtRule;
export const PostCSSDeclaration = postcss.Declaration;
export const PostCSSComment = postcss.Comment;
export const PostCSSContainer = postcss.Container;
export const PostCSSNode = postcss.Node;

export type {
  Root,
  Rule,
  AtRule,
  Declaration,
  Comment,
  Container,
  Node,
  Plugin,
  Processor,
} from 'postcss';
