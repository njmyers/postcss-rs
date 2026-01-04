#!/usr/bin/env node

/**
 * Basic tests for @postcss-rs/node-postcss
 */

const { parse } = require('../src/index');
const postcss = require('postcss');
const assert = require('assert');

async function runTests() {
  console.log('🧪 Running Basic Tests\n');
  console.log('='.repeat(60));

  let passed = 0;
  let failed = 0;

  async function test(name, fn) {
    try {
      await fn();
      console.log(`✅ ${name}`);
      passed++;
    } catch (err) {
      console.error(`❌ ${name}`);
      console.error(`   Error: ${err.message}`);
      failed++;
    }
  }

  // Test 1: Basic parsing
  await test('Parse simple CSS', async () => {
    const root = await parse('.test { color: red; }');
    assert.strictEqual(root.type, 'root');
    assert.strictEqual(root.nodes.length, 1);
    assert.strictEqual(root.first.type, 'rule');
    assert.strictEqual(root.first.selector, '.test');
  });

  // Test 2: PostCSS class instances
  await test('Returns real PostCSS class instances', async () => {
    const root = await parse('.test { color: red; }');
    assert.strictEqual(root.constructor.name, 'Root');
    assert.strictEqual(root.first.constructor.name, 'Rule');
    assert.strictEqual(root.first.first.constructor.name, 'Declaration');
  });

  // Test 3: walkDecls works
  await test('walkDecls() method works', async () => {
    const root = await parse('.test { color: red; font-size: 14px; }');
    const decls = [];
    root.walkDecls(decl => {
      decls.push(decl.prop);
    });
    assert.deepStrictEqual(decls, ['color', 'font-size']);
  });

  // Test 4: Mutations work
  await test('Mutations work correctly', async () => {
    const root = await parse('.test { color: red; }');
    root.first.first.value = 'blue';
    assert.strictEqual(root.first.first.value, 'blue');
  });

  // Test 5: Clone works
  await test('clone() method works', async () => {
    const root = await parse('.test { color: red; }');
    const cloned = root.clone();
    assert.notStrictEqual(root, cloned);
    assert.strictEqual(root.toString(), cloned.toString());
  });

  // Test 6: toString() works
  await test('toString() preserves formatting', async () => {
    const css = '.test { color: red; }';
    const root = await parse(css);
    assert.strictEqual(root.toString(), css);
  });

  // Test 7: At-rules
  await test('Parses at-rules correctly', async () => {
    const css = '@media screen { .test { color: red; } }';
    const root = await parse(css);
    assert.strictEqual(root.first.type, 'atrule');
    assert.strictEqual(root.first.name, 'media');
    assert.strictEqual(root.first.params, 'screen ');
  });

  // Test 8: Comments
  await test('Parses comments correctly', async () => {
    const css = '/* comment */ .test { color: red; }';
    const root = await parse(css);
    assert.strictEqual(root.first.type, 'comment');
    assert.strictEqual(root.first.text, ' comment ');
  });

  // Test 9: Important declarations
  await test('Handles !important correctly', async () => {
    const css = '.test { color: red !important; }';
    const root = await parse(css);
    assert.strictEqual(root.first.first.important, true);
  });

  // Test 10: Multiple selectors
  await test('Handles multiple selectors', async () => {
    const css = '.a, .b { color: red; }';
    const root = await parse(css);
    assert.strictEqual(root.first.selector, '.a, .b');
    assert.deepStrictEqual(root.first.selectors, ['.a', '.b']);
  });

  // Test 11: Works with autoprefixer plugin
  await test('Works with autoprefixer plugin', async () => {
    const autoprefixer = require('autoprefixer');
    const root = await parse('.test { display: flex; }');
    const result = await postcss([autoprefixer]).process(root, { from: undefined });
    assert(result.css.includes('display'));
  });

  console.log('\n' + '='.repeat(60));
  console.log(`\nResults: ${passed} passed, ${failed} failed`);

  if (failed > 0) {
    process.exit(1);
  } else {
    console.log('\n🎉 All tests passed!\n');
  }
}

runTests().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});
