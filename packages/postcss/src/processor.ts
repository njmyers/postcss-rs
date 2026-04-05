import * as postcss from 'postcss';
import type {
  Plugin,
  ProcessOptions,
  Processor,
  Root,
  Stringifier,
  Syntax,
} from 'postcss';

type SyncParseFn = (css: string, opts?: any) => Root;
type AsyncParseFn = (css: string, opts?: any) => Promise<Root>;
export type ParseFn = AsyncParseFn | SyncParseFn;

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
 * import { RustProcessorFactory, parseOffsetsAsync } from '@postcss-rs/postcss';
 *
 * const OffsetProcessor = RustProcessorFactory(parseOffsetsAsync);
 * const result = await new OffsetProcessor().process(css, { from: undefined }).async();
 * ```
 */
export function RustProcessorFactory(
  parseFn: ParseFn,
  stringifyFn: Stringifier | Syntax = postcss.stringify
): new (plugins?: Plugin[]) => Processor {
  class RustProcessor extends postcss.Processor {
    constructor(plugins: Plugin[] = []) {
      super(plugins);
    }

    process(css: Root | string, opts: ProcessOptions = {}): any {
      if (typeof css !== 'string') {
        return super.process(css, {
          stringifier: stringifyFn as Stringifier,
          ...opts,
        });
      }

      const mergedOpts: ProcessOptions = {
        stringifier: stringifyFn as Stringifier,
        ...opts,
      };

      const parsed = parseFn(css, opts);

      if (parsed instanceof Promise) {
        return {
          async: () =>
            parsed.then((root: Root) =>
              super.process(root, mergedOpts).async()
            ),
          sync: () => {
            throw new Error(
              'sync() is not supported when using an async parse function'
            );
          },
        };
      }

      return super.process(parsed, mergedOpts);
    }
  }

  return RustProcessor;
}
