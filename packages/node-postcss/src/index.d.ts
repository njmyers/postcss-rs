/**
 * TypeScript definitions for @postcss-rs/node-postcss
 */

import {
  Root,
  Processor,
  Plugin,
  ProcessOptions,
  Parser,
  Node as PostCSSNode,
  Container as PostCSSContainer,
  Rule as PostCSSRule,
  AtRule as PostCSSAtRule,
  Declaration as PostCSSDeclaration,
  Comment as PostCSSComment
} from 'postcss';

/**
 * Parse CSS into a PostCSS AST
 *
 * This function uses a Rust-based parser for maximum performance (3-5x faster),
 * then hydrates the result into actual PostCSS class instances for full API compatibility.
 *
 * @param css - CSS source code to parse
 * @param opts - Parse options
 * @returns PostCSS Root instance
 *
 * @example
 * ```typescript
 * import { parse } from '@postcss-rs/node-postcss';
 *
 * const root = await parse('.test { color: red; }', { from: 'test.css' });
 * console.log(root.first.selector); // '.test'
 * ```
 */
export function parse(css: string, opts?: ProcessOptions): Promise<Root>;

/**
 * Synchronous parse (not yet implemented)
 *
 * @param css - CSS source code to parse
 * @param opts - Parse options
 * @returns PostCSS Root instance
 * @throws Error indicating sync parsing is not yet supported
 */
export function parseSync(css: string, opts?: ProcessOptions): Root;

/**
 * Create a PostCSS processor with Rust-powered parsing
 *
 * @param plugins - PostCSS plugins to use
 * @returns PostCSS Processor with Rust parser
 *
 * @example
 * ```typescript
 * import postcssRs from '@postcss-rs/node-postcss';
 * import autoprefixer from 'autoprefixer';
 *
 * const processor = postcssRs([autoprefixer]);
 * const result = await processor.process(css, { from: 'input.css' });
 * ```
 */
declare function postcssRs(plugins?: Plugin[]): Processor;

export default postcssRs;

// Re-export PostCSS types for convenience
export {
  Root,
  Rule,
  AtRule,
  Declaration,
  Comment,
  Container,
  Node,
  Processor,
  Plugin,
  ProcessOptions
} from 'postcss';

// Re-export the postcss namespace
export { postcss } from 'postcss';

/**
 * PostCSS Root class
 */
export const Root: typeof PostCSSNode;

/**
 * PostCSS Rule class
 */
export const Rule: typeof PostCSSRule;

/**
 * PostCSS AtRule class
 */
export const AtRule: typeof PostCSSAtRule;

/**
 * PostCSS Declaration class
 */
export const Declaration: typeof PostCSSDeclaration;

/**
 * PostCSS Comment class
 */
export const Comment: typeof PostCSSComment;

/**
 * PostCSS Container class
 */
export const Container: typeof PostCSSContainer;

/**
 * PostCSS Node class
 */
export const Node: typeof PostCSSNode;
