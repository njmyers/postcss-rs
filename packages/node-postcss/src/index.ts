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

import { parseOffsetsSync } from './parsers/offset';
import { RustProcessorFactory } from './processor';

export type { ParseOptions } from './parsers/json';
export { parseJsonAsync, parseJsonSync } from './parsers/json';
export {
  parseOffsetsAsync,
  parseOffsetsRustOnlySync,
  parseOffsetsSync,
} from './parsers/offset';
export { RustProcessorFactory } from './processor';
export type { ParseFn } from './processor';

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
  AtRule,
  Comment,
  Container,
  Declaration,
  Node,
  Plugin,
  Processor,
  Root,
  Rule,
} from 'postcss';
