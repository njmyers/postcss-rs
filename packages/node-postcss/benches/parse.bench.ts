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
import { readFileSync } from 'fs';
import { join } from 'path';
import {
  RustProcessorFactory,
  parseJsonAsync,
  parseJsonSync,
  parseOffsetsSync,
  parseOffsetsAsync,
} from '../src/index';
import postcss from 'postcss';

const SMALL_CSS = readFileSync(join(__dirname, 'fixtures/small.css'), 'utf8');
const MEDIUM_CSS = readFileSync(join(__dirname, 'fixtures/medium.css'), 'utf8');
const LARGE_CSS = readFileSync(join(__dirname, 'fixtures/large.css'), 'utf8');

// Build processors once — factory returns a class, instantiate per suite call
// so each benchmark gets a fresh processor with no shared state.
const JsonSyncProcessor = RustProcessorFactory(parseJsonSync);
const JsonAsyncProcessor = RustProcessorFactory(parseJsonAsync as any);
const OffsetSyncProcessor = RustProcessorFactory(parseOffsetsSync);
const OffsetAsyncProcessor = RustProcessorFactory(parseOffsetsAsync as any);

async function makeSuite(name: string, css: string) {
  return b.suite(
    name,
    b.add('postcss-rs JSON sync', async () => {
      (await new JsonSyncProcessor().process(css, { from: undefined }).async()).root;
    }),
    b.add('postcss-rs JSON async', async () => {
      (await new JsonAsyncProcessor().process(css, { from: undefined }).async()).root;
    }),
    b.add('postcss-rs Offsets sync', async () => {
      (await new OffsetSyncProcessor().process(css, { from: undefined }).async()).root;
    }),
    b.add('postcss-rs Offsets async', async () => {
      (await new OffsetAsyncProcessor().process(css, { from: undefined }).async()).root;
    }),
    b.add('postcss (JS)', async () => {
      (await postcss().process(css, { from: undefined }).async()).root;
    }),
    b.cycle(),
    b.complete()
  );
}

makeSuite('Small (~200 B)', SMALL_CSS)
  .then(() => makeSuite('Medium (~2 KB)', MEDIUM_CSS))
  .then(() => makeSuite('Large (~30 KB)', LARGE_CSS))
  .catch((err: Error) => {
    console.error(err);
    process.exit(1);
  });
