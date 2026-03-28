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
import type { Root, Plugin, Processor, ProcessOptions, Stringifier, Syntax } from 'postcss';

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

type SyncParseFn = (css: string, opts?: any) => Root;
type AsyncParseFn = (css: string, opts?: any) => Promise<Root>;
type ParseFn = SyncParseFn | AsyncParseFn;

/**
 * Returns a PostCSS Processor class that uses the provided `parseFn` and
 * `stringifyFn` by default on every `.process()` call.
 *
 * Works with both synchronous and asynchronous parse functions:
 * - Sync parseFn: returned processor supports both `.sync()` and `.async()`
 * - Async parseFn: returned processor only supports `.async()` (`.sync()` throws)
 *
 * The parse result (Root) is passed directly to PostCSS, bypassing its own
 * parser so the full plugin chain and stringify still run normally.
 *
 * @example
 * ```ts
 * import { RustProcessorFactory, parseOffsetsAsync } from '@postcss-rs/node-postcss';
 *
 * const OffsetProcessor = RustProcessorFactory(parseOffsetsAsync);
 * const result = await new OffsetProcessor().process(css, { from: undefined }).async();
 * ```
 */
export function RustProcessorFactory(
  parseFn: ParseFn,
  stringifyFn: Stringifier | Syntax = postcss.stringify,
): new (plugins?: Plugin[]) => Processor {
  class RustProcessor extends postcss.Processor {
    constructor(plugins: Plugin[] = []) {
      super(plugins);
    }

    process(css: string | Root, opts: ProcessOptions = {}): any {
      if (typeof css !== 'string') {
        return super.process(css, { stringifier: stringifyFn as Stringifier, ...opts });
      }

      const mergedOpts: ProcessOptions = {
        stringifier: stringifyFn as Stringifier,
        ...opts,
      };

      const parsed = parseFn(css, opts);

      if (parsed instanceof Promise) {
        return {
          async: () =>
            parsed.then((root: Root) => super.process(root, mergedOpts).async()),
          sync: () => {
            throw new Error(
              'sync() is not supported when using an async parse function',
            );
          },
        };
      }

      return super.process(parsed as Root, mergedOpts);
    }
  }

  return RustProcessor;
}

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
export { postcss };

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
