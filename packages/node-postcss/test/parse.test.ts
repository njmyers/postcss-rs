/**
 * Basic tests for @postcss-rs/node-postcss
 */

import {
  parseJsonAsync,
  parseJsonSync,
  parseOffsetsSync,
  parseOffsetsAsync,
} from '../src/index';
import * as postcss from 'postcss';
import type { Root, Rule, Declaration } from 'postcss';
import { describe, test, expect } from 'vitest';

type ParseFn = (css: string) => Root | Promise<Root>;

const parsers: Array<[string, ParseFn]> = [
  ['parseJsonAsync', css => parseJsonAsync(css)],
  ['parseJsonSync', css => parseJsonSync(css)],
  ['parseOffsetsSync', css => parseOffsetsSync(css)],
  ['parseOffsetsAsync', css => parseOffsetsAsync(css)],
];

describe.each(parsers)('@postcss-rs/node-postcss (%s)', (_name, parse) => {
  test('Parse simple CSS', async () => {
    const root = await parse('.test { color: red; }');
    expect(root.type).toBe('root');
    expect(root.nodes.length).toBe(1);
    expect(root.first?.type).toBe('rule');
    expect((root.first as Rule)?.selector).toBe('.test');
  });

  test('Returns real PostCSS class instances', async () => {
    const root = await parse('.test { color: red; }');
    expect(root.constructor.name).toBe('Root');
    expect(root.first?.constructor.name).toBe('Rule');
    expect((root.first as Rule)?.first?.constructor.name).toBe('Declaration');
  });

  test('walkDecls() method works', async () => {
    const root = await parse('.test { color: red; font-size: 14px; }');
    const decls: string[] = [];
    root.walkDecls(decl => {
      decls.push(decl.prop);
    });
    expect(decls).toEqual(['color', 'font-size']);
  });

  test('Mutations work correctly', async () => {
    const root = await parse('.test { color: red; }');
    const firstDecl = (root.first as Rule)?.first as Declaration;
    firstDecl.value = 'blue';
    expect(firstDecl.value).toBe('blue');
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
    expect(root.first?.type).toBe('atrule');
    expect((root.first as any)?.name).toBe('media');
    expect((root.first as any)?.params).toBe('screen');
  });

  test('Parses comments correctly', async () => {
    const css = '/* comment */ .test { color: red; }';
    const root = await parse(css);
    expect(root.first?.type).toBe('comment');
    expect((root.first as any)?.text).toBe('comment');
  });

  test('Handles !important correctly', async () => {
    const css = '.test { color: red !important; }';
    const root = await parse(css);
    const firstDecl = (root.first as Rule)?.first as Declaration;
    expect(firstDecl.important).toBe(true);
  });

  test('Handles multiple selectors', async () => {
    const css = '.a, .b { color: red; }';
    const root = await parse(css);
    const rule = root.first as Rule;
    expect(rule.selector).toBe('.a, .b');
    expect(rule.selectors).toEqual(['.a', '.b']);
  });

  test('Works with autoprefixer plugin', async () => {
    const autoprefixer = require('autoprefixer');
    const root = await parse('.test { display: flex; }');
    const result = await postcss
      .default([autoprefixer])
      .process(root, { from: undefined });
    expect(result.css).toContain('display');
  });
});
