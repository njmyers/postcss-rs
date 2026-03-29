/**
 * Benchmarks comparing @postcss-rs/node-postcss (Rust) vs postcss (JS)
 *
 * Run with: pnpm bench
 *
 * All suites use the real-world async Processor path:
 *   new Processor().process(css, opts).async()
 *
 * This exercises the full pipeline — parse, plugin chain, stringify setup —
 * under the same Promise/event-loop conditions as production usage.
 */

import b from 'benny';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import postcss, { type Processor } from 'postcss';

import {
  parseJsonAsync,
  parseJsonSync,
  parseOffsetsAsync,
  parseOffsetsSync,
  RustProcessorFactory,
} from '../src/index';

const SMALL_CSS = readFileSync(join(__dirname, 'fixtures/small.css'), 'utf8');
const MEDIUM_CSS = readFileSync(join(__dirname, 'fixtures/medium.css'), 'utf8');
const LARGE_CSS = readFileSync(join(__dirname, 'fixtures/large.css'), 'utf8');

// Build processors once — factory returns a class, instantiate per suite call
// so each benchmark gets a fresh processor with no shared state.
const JsonSyncProcessor = RustProcessorFactory(parseJsonSync);
const JsonAsyncProcessor = RustProcessorFactory(parseJsonAsync);
const OffsetSyncProcessor = RustProcessorFactory(parseOffsetsSync);
const OffsetAsyncProcessor = RustProcessorFactory(parseOffsetsAsync);

async function makeSuite(name: string, css: string) {
  return b.suite(
    name,
    b.add('postcss-rs json sync', async () => {
      await run(new JsonSyncProcessor(), css);
    }),
    b.add('postcss-rs json async', async () => {
      await run(new JsonAsyncProcessor(), css);
    }),
    b.add('postcss-rs offsets sync', async () => {
      await run(new OffsetSyncProcessor(), css);
    }),
    b.add('postcss-rs offsets async', async () => {
      await run(new OffsetAsyncProcessor(), css);
    }),
    b.add('postcss original', async () => {
      await run(postcss(), css);
    }),
    b.cycle(),
    b.complete()
  );
}

async function run(processor: Processor, css: string) {
  const result = await processor.process(css, { from: undefined }).async();
  return result.root;
}

(async () => {
  try {
    await makeSuite('Small (~200 B)', SMALL_CSS);
    await makeSuite('Medium (~2 KB)', MEDIUM_CSS);
    await makeSuite('Large (~30 KB)', LARGE_CSS);
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
