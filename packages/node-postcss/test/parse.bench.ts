/**
 * Benchmarks comparing @postcss-rs/node-postcss (Rust) vs postcss (JS)
 *
 * Run with: pnpm bench
 */

import b from 'benny';
import {
  parse, parseSync, parseFb, parseFbSync,
  parseMsgpack, parseMsgpackSync,
  parseRustOnlySync, parseJsonOnlySync,
  parseFbRustOnlySync, parseFbDecodeOnlySync,
  parseMsgpackRustOnlySync, parseMsgpackDecodeOnlySync,
} from '../src/index';
import postcss from 'postcss';
import { unpack as msgpackrUnpack } from 'msgpackr';

// ─── CSS Fixtures ────────────────────────────────────────────────────────────

const SMALL_CSS = `.button {
  display: inline-flex;
  align-items: center;
  padding: 8px 16px;
  background-color: #0070f3;
  color: #fff;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}`;

const MEDIUM_CSS = `
/* Reset */
*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

:root {
  --color-primary: #0070f3;
  --color-secondary: #7928ca;
  --color-background: #fff;
  --color-text: #111;
  --font-size-base: 16px;
  --spacing-unit: 8px;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, sans-serif;
  font-size: var(--font-size-base);
  color: var(--color-text);
  background: var(--color-background);
  line-height: 1.5;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 calc(var(--spacing-unit) * 2);
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: calc(var(--spacing-unit) * 1) calc(var(--spacing-unit) * 2);
  border: none;
  border-radius: 4px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s ease, transform 0.1s ease;
}

.btn-primary {
  background-color: var(--color-primary);
  color: #fff;
}

.btn-primary:hover {
  background-color: #0060df;
}

.btn-primary:active {
  transform: scale(0.97);
}

.card {
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: calc(var(--spacing-unit) * 3);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.card__title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: calc(var(--spacing-unit) * 1);
}

.card__body {
  color: #6b7280;
  line-height: 1.6;
}

@media (max-width: 768px) {
  .container {
    padding: 0 var(--spacing-unit);
  }

  .card {
    border-radius: 4px;
  }
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-8px); }
  to   { opacity: 1; transform: translateY(0); }
}

@supports (display: grid) {
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: calc(var(--spacing-unit) * 3);
  }
}
`;

// Build a large CSS string by repeating component patterns with unique class names
const LARGE_CSS = Array.from({ length: 50 }, (_, i) => `
.component-${i} {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: ${8 + i}px ${16 + i}px;
  margin: ${4 + i}px;
  background-color: hsl(${i * 7}, 60%, 95%);
  border: 1px solid hsl(${i * 7}, 40%, 80%);
  border-radius: ${4 + (i % 8)}px;
  font-size: ${12 + (i % 6)}px;
  color: hsl(${i * 7}, 40%, 20%);
  transition: all 0.2s ease;
}

.component-${i}:hover {
  background-color: hsl(${i * 7}, 60%, 88%);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.component-${i}__title {
  font-size: ${14 + (i % 4)}px;
  font-weight: 600;
  margin-bottom: 8px;
}

@media (max-width: ${320 + i * 10}px) {
  .component-${i} {
    padding: 8px;
    font-size: 12px;
  }
}
`).join('\n');

const processor = postcss();

// ─── Suite factories ──────────────────────────────────────────────────────────

function makeRawSuite(name: string, css: string) {
  return b.suite(
    `${name} — raw (Rust only, no tree walk)`,
    b.add('JSON string only',     () => { parseRustOnlySync(css); }),
    b.add('JSON.parse only',      () => { parseJsonOnlySync(css); }),
    b.add('FB buffer only',       () => { parseFbRustOnlySync(css); }),
    b.add('FB decode only',       () => { parseFbDecodeOnlySync(css); }),
    b.add('Msgpack buffer only',  () => { parseMsgpackRustOnlySync(css); }),
    b.add('Msgpack decode only',  () => { parseMsgpackDecodeOnlySync(css); }),
    b.add('Msgpackr decode only', () => { msgpackrUnpack(parseMsgpackRustOnlySync(css)); }),
    b.cycle(),
    b.complete(),
  );
}

function makeE2eSuite(name: string, css: string) {
  return b.suite(
    `${name} — e2e (full parse → PostCSS AST)`,
    b.add('postcss-rs JSON sync',     () => { parseSync(css); }),
    b.add('postcss-rs FB sync',       () => { parseFbSync(css); }),
    b.add('postcss-rs Msgpack sync',  () => { parseMsgpackSync(css); }),
    b.add('postcss-rs JSON async',    async () => { await parse(css); }),
    b.add('postcss-rs FB async',      async () => { await parseFb(css); }),
    b.add('postcss-rs Msgpack async', async () => { await parseMsgpack(css); }),
    b.add('postcss sync (JS)',         () => { processor.process(css, { from: undefined }).sync().root; }),
    b.add('postcss async (JS)',        async () => { (await processor.process(css, { from: undefined }).async()).root; }),
    b.cycle(),
    b.complete(),
  );
}

// ─── Run suites sequentially ─────────────────────────────────────────────────

makeRawSuite('Small (~200 B)', SMALL_CSS)
  .then(() => makeE2eSuite('Small (~200 B)', SMALL_CSS))
  .then(() => makeRawSuite('Medium (~2 KB)', MEDIUM_CSS))
  .then(() => makeE2eSuite('Medium (~2 KB)', MEDIUM_CSS))
  .then(() => makeRawSuite('Large (~20 KB)', LARGE_CSS))
  .then(() => makeE2eSuite('Large (~20 KB)', LARGE_CSS))
  .catch((err: Error) => { console.error(err); process.exit(1); });
