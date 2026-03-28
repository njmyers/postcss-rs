/**
 * Basic tests for @postcss-rs/node-postcss
 */

import * as postcss from 'postcss';
import type { Declaration, Root, Rule } from 'postcss';
import { describe, expect, test } from 'vitest';

import {
  parseJsonAsync,
  parseJsonSync,
  parseOffsetsAsync,
  parseOffsetsSync,
} from '../src/index';

type ParseFn = (css: string) => Promise<Root> | Root;

const parsers: [string, ParseFn][] = [
  ['parseJsonAsync', css => parseJsonAsync(css)],
  ['parseJsonSync', css => parseJsonSync(css)],
  ['parseOffsetsSync', css => parseOffsetsSync(css)],
  ['parseOffsetsAsync', css => parseOffsetsAsync(css)],
];

describe.each(parsers)('@postcss-rs/node-postcss (%s)', (_name, parse) => {
  test('Parse simple CSS', async () => {
    let root = await parse('.test { color: red; }');
    expect(root.type).toBe('root');
    expect(root.nodes.length).toBe(1);
    expect(root.first?.type).toBe('rule');
    expect((root.first as Rule).selector).toBe('.test');
  });

  test('Returns real PostCSS class instances', async () => {
    let root = await parse('.test { color: red; }');
    expect(root.constructor.name).toBe('Root');
    expect(root.first?.constructor.name).toBe('Rule');
    expect((root.first as Rule).first?.constructor.name).toBe('Declaration');
  });

  test('walkDecls() method works', async () => {
    let root = await parse('.test { color: red; font-size: 14px; }');
    let decls: string[] = [];
    root.walkDecls(decl => {
      decls.push(decl.prop);
    });
    expect(decls).toEqual(['color', 'font-size']);
  });

  test('Mutations work correctly', async () => {
    let root = await parse('.test { color: red; }');
    let firstDecl = (root.first as Rule).first as Declaration;
    firstDecl.value = 'blue';
    expect(firstDecl.value).toBe('blue');
  });

  test('clone() method works', async () => {
    let root = await parse('.test { color: red; }');
    let cloned = root.clone();
    expect(root).not.toBe(cloned);
    expect(root.toString()).toBe(cloned.toString());
  });

  test('toString() preserves formatting', async () => {
    let css = '.test { color: red; }';
    let root = await parse(css);
    expect(root.toString()).toBe(css);
  });

  test('Parses at-rules correctly', async () => {
    let css = '@media screen { .test { color: red; } }';
    let root = await parse(css);
    expect(root.first?.type).toBe('atrule');
    expect((root.first as any)?.name).toBe('media');
    expect((root.first as any)?.params).toBe('screen');
  });

  test('Parses comments correctly', async () => {
    let css = '/* comment */ .test { color: red; }';
    let root = await parse(css);
    expect(root.first?.type).toBe('comment');
    expect((root.first as any)?.text).toBe('comment');
  });

  test('Handles !important correctly', async () => {
    let css = '.test { color: red !important; }';
    let root = await parse(css);
    let firstDecl = (root.first as Rule).first as Declaration;
    expect(firstDecl.important).toBe(true);
  });

  test('Handles multiple selectors', async () => {
    let css = '.a, .b { color: red; }';
    let root = await parse(css);
    let rule = root.first as Rule;
    expect(rule.selector).toBe('.a,.b');
    expect(rule.selectors).toEqual(['.a', '.b']);
  });

  test('Works with autoprefixer plugin', async () => {
    let autoprefixer = require('autoprefixer');
    let root = await parse('.test { display: flex; }');
    let result = await postcss
      .default([autoprefixer])
      .process(root, { from: undefined });
    expect(result.css).toContain('display');
  });
});
