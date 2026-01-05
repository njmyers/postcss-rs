/**
 * Basic tests for @postcss-rs/node-postcss
 */

const { parse } = require('../src/index');
const postcss = require('postcss');

describe('@postcss-rs/node-postcss', () => {
  test('Parse simple CSS', async () => {
    const root = await parse('.test { color: red; }');
    expect(root.type).toBe('root');
    expect(root.nodes.length).toBe(1);
    expect(root.first.type).toBe('rule');
    expect(root.first.selector).toBe('.test');
  });

  test('Returns real PostCSS class instances', async () => {
    const root = await parse('.test { color: red; }');
    expect(root.constructor.name).toBe('Root');
    expect(root.first.constructor.name).toBe('Rule');
    expect(root.first.first.constructor.name).toBe('Declaration');
  });

  test('walkDecls() method works', async () => {
    const root = await parse('.test { color: red; font-size: 14px; }');
    const decls = [];
    root.walkDecls(decl => {
      decls.push(decl.prop);
    });
    expect(decls).toEqual(['color', 'font-size']);
  });

  test('Mutations work correctly', async () => {
    const root = await parse('.test { color: red; }');
    root.first.first.value = 'blue';
    expect(root.first.first.value).toBe('blue');
  });

  test('clone() method works', async () => {
    const root = await parse('.test { color: red; }');
    const cloned = root.clone();
    expect(root).not.toBe(cloned);
    expect(root.toString()).toBe(cloned.toString());
  });

  test('toString() preserves formatting', async () => {
    const css = '.test { color: red; }';
    const root = await parse(css);
    expect(root.toString()).toBe(css);
  });

  test('Parses at-rules correctly', async () => {
    const css = '@media screen { .test { color: red; } }';
    const root = await parse(css);
    expect(root.first.type).toBe('atrule');
    expect(root.first.name).toBe('media');
    expect(root.first.params).toBe('screen');
  });

  test('Parses comments correctly', async () => {
    const css = '/* comment */ .test { color: red; }';
    const root = await parse(css);
    expect(root.first.type).toBe('comment');
    expect(root.first.text).toBe('comment');
  });

  test('Handles !important correctly', async () => {
    const css = '.test { color: red !important; }';
    const root = await parse(css);
    expect(root.first.first.important).toBe(true);
  });

  test('Handles multiple selectors', async () => {
    const css = '.a, .b { color: red; }';
    const root = await parse(css);
    expect(root.first.selector).toBe('.a,.b');
    expect(root.first.selectors).toEqual(['.a', '.b']);
  });

  test('Works with autoprefixer plugin', async () => {
    const autoprefixer = require('autoprefixer');
    const root = await parse('.test { display: flex; }');
    const result = await postcss([autoprefixer]).process(root, { from: undefined });
    expect(result.css).toContain('display');
  });
});
