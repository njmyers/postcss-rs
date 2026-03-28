/**
 * Benchmarks comparing @postcss-rs/node-postcss (Rust) vs postcss (JS)
 *
 * Run with: pnpm bench
 *
 * Suites are grouped by operation type so related variants are compared
 * directly:
 *
 *   Sync  — blocking parse to PostCSS AST
 *   Async — non-blocking parse to PostCSS AST
 *   Raw   — Rust serialisation only (no JS tree walk)
 */

import b from 'benny';
import { readFileSync } from 'fs';
import { join } from 'path';
import {
  parseJsonAsync,
  parseJsonSync,
  parseOffsetsSync,
  parseOffsetsAsync,
  parseRustOnlySync,
  parseJsonOnlySync,
  parseOffsetsRustOnlySync,
} from '../src/index';
import postcss from 'postcss';

const SMALL_CSS = readFileSync(join(__dirname, 'fixtures/small.css'), 'utf8');
const MEDIUM_CSS = readFileSync(join(__dirname, 'fixtures/medium.css'), 'utf8');
const LARGE_CSS = readFileSync(join(__dirname, 'fixtures/large.css'), 'utf8');

const processor = postcss();

function makeSyncSuite(name: string, css: string) {
  return b.suite(
    `${name} — sync`,
    b.add('postcss-rs JSON', () => {
      parseJsonSync(css);
    }),
    b.add('postcss-rs Offsets', () => {
      parseOffsetsSync(css);
    }),
    b.add('postcss (JS)', () => {
      processor.process(css, { from: undefined }).sync().root;
    }),
    b.cycle(),
    b.complete()
  );
}

function makeAsyncSuite(name: string, css: string) {
  return b.suite(
    `${name} — async`,
    b.add('postcss-rs JSON', async () => {
      await parseJsonAsync(css);
    }),
    b.add('postcss-rs Offsets', async () => {
      await parseOffsetsAsync(css);
    }),
    b.add('postcss (JS)', async () => {
      (await processor.process(css, { from: undefined }).async()).root;
    }),
    b.cycle(),
    b.complete()
  );
}

function makeRawSuite(name: string, css: string) {
  return b.suite(
    `${name} — raw (Rust serialisation only, no JS tree walk)`,
    b.add('JSON string', () => {
      parseRustOnlySync(css);
    }),
    b.add('JSON.parse', () => {
      parseJsonOnlySync(css);
    }),
    b.add('Offsets buffer', () => {
      parseOffsetsRustOnlySync(css);
    }),
    b.cycle(),
    b.complete()
  );
}

makeSyncSuite('Small (~200 B)', SMALL_CSS)
  .then(() => makeSyncSuite('Medium (~2 KB)', MEDIUM_CSS))
  .then(() => makeSyncSuite('Large (~30 KB)', LARGE_CSS))
  .then(() => makeAsyncSuite('Small (~200 B)', SMALL_CSS))
  .then(() => makeAsyncSuite('Medium (~2 KB)', MEDIUM_CSS))
  .then(() => makeAsyncSuite('Large (~30 KB)', LARGE_CSS))
  .then(() => makeRawSuite('Small (~200 B)', SMALL_CSS))
  .then(() => makeRawSuite('Medium (~2 KB)', MEDIUM_CSS))
  .then(() => makeRawSuite('Large (~30 KB)', LARGE_CSS))
  .catch((err: Error) => {
    console.error(err);
    process.exit(1);
  });
