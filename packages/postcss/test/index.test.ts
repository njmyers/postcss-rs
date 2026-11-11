/**
 * Tests for packages/postcss/src/index.ts
 * Includes tests for parse(), restoreRaws(), and PostCSS integration
 */

import { describe, expect, test } from 'vitest';

import { parseJsonAsync as parse } from '../src/index';

describe('parse() function', () => {
  test('Returns PostCSS Root instance', async () => {
    const root = await parse('.test { color: red; }');
    expect(root.type).toBe('root');
    expect(root.constructor.name).toBe('Root');
  });

  test('Attaches source.input to all nodes', async () => {
    const css = '.test { color: red; }';
    const root = await parse(css, { from: 'test.css' });

    // Check root has input
    expect(root.source?.input).toBeDefined();
    expect(root.source?.input.from).toContain('test.css');

    // Check rule has input
    const rule = root.first;
    expect(rule?.source?.input).toBeDefined();
    expect(rule?.source?.input.from).toContain('test.css');

    // Check declaration has input
    const decl = (rule as any)?.first;
    expect(decl?.source?.input).toBeDefined();
    expect(decl?.source?.input.from).toContain('test.css');
  });

  test('Accepts from option', async () => {
    const root = await parse('.test { color: red; }', { from: 'styles.css' });
    expect(root.source?.input.from).toContain('styles.css');
  });

  test('Works without options', async () => {
    const root = await parse('.test { color: red; }');
    expect(root.source?.input).toBeDefined();
  });
});

describe('restoreRaws() - Whitespace preservation', () => {
  test('Preserves consecutive newlines in raws.before', async () => {
    const css = '/* comment */\n\n.test { color: red; }';
    const root = await parse(css);

    // The rule after the comment should have "\n\n" in raws.before
    const rule = root.nodes.find(n => n.type === 'rule') as any;
    expect(rule?.raws?.before).toBe('\n\n');
  });

  test('Preserves multiple newlines', async () => {
    const css = '/* a */\n\n\n.test { color: red; }';
    const root = await parse(css);

    const rule = root.nodes.find(n => n.type === 'rule') as any;
    expect(rule?.raws?.before).toBe('\n\n\n');
  });

  test('Preserves whitespace in nested structures', async () => {
    const css = `@media screen {
  .test { color: red; }
  .test2 { color: blue; }
}`;
    const root = await parse(css);

    const mediaRule = root.first as any;
    const secondRule = mediaRule?.nodes?.[1];

    // Second rule should have whitespace before it
    expect(secondRule?.raws?.before).toBeDefined();
    expect(secondRule?.raws?.before).toContain('\n');
  });

  test('Handles empty raws.before correctly', async () => {
    const css = '.test { color: red; }';
    const root = await parse(css);

    const rule = root.first as any;
    expect(rule?.raws?.before).toBe('');
  });

  test('Preserves raws.after whitespace', async () => {
    const css = '.test { color: red;\n  }';
    const root = await parse(css);

    const rule = root.first as any;
    expect(rule?.raws?.after).toContain('\n');
  });

  test('Preserves raws.between whitespace', async () => {
    const css = '.test   { color: red; }';
    const root = await parse(css);

    const rule = root.first as any;
    expect(rule?.raws?.between).toBe('   ');
  });

  test('Preserves declaration whitespace', async () => {
    const css = '.test { color  :  red ; }';
    const root = await parse(css);

    const decl = (root.first as any)?.first;
    expect(decl?.raws?.between).toBe('  :  ');
  });

  test('Preserves comment whitespace', async () => {
    const css = '/*  text  */';
    const root = await parse(css);

    const comment = root.first as any;
    expect(comment?.raws?.left).toBe('  ');
    expect(comment?.raws?.right).toBe('  ');
  });
});

describe('PostCSS compatibility', () => {
  test('Nodes can be stringified correctly', async () => {
    const css = '.test { color: red; }';
    const root = await parse(css);
    expect(root.toString()).toBe(css);
  });

  test('toJSON() includes all expected fields', async () => {
    const css = '/* a */\n\n.test { color: red; }';
    const root = await parse(css);

    const json = root.toJSON();
    expect(json.type).toBe('root');
    expect(json.nodes).toBeDefined();
    expect(Array.isArray(json.nodes)).toBe(true);
  });

  test('Rule nodes have correct structure', async () => {
    const css = '.test { color: red; }';
    const root = await parse(css);
    const rule = root.first as any;

    const json = rule.toJSON();
    expect(json.type).toBe('rule');
    expect(json.selector).toBe('.test');
    expect(json.nodes).toBeDefined();
    expect(json.raws).toBeDefined();
  });

  test('At-rule nodes have correct structure', async () => {
    const css = '@media screen { }';
    const root = await parse(css);
    const atRule = root.first as any;

    expect(atRule.type).toBe('atrule');
    expect(atRule.name).toBe('media');
    expect(atRule.params).toBe('screen');
    expect(atRule.nodes).toBeDefined();
  });

  test('Declaration nodes have correct structure', async () => {
    const css = '.test { color: red; }';
    const root = await parse(css);
    const decl = (root.first as any)?.first;

    const json = decl.toJSON();
    expect(json.type).toBe('decl');
    expect(json.prop).toBe('color');
    expect(json.value).toBe('red');
    expect(json.nodes).toBeUndefined(); // Declarations don't have nodes
  });

  test('Comment nodes have correct structure', async () => {
    const css = '/* comment */';
    const root = await parse(css);
    const comment = root.first as any;

    const json = comment.toJSON();
    expect(json.type).toBe('comment');
    expect(json.text).toBe('comment');
    expect(json.nodes).toBeUndefined(); // Comments don't have nodes
  });
});

describe('Edge cases', () => {
  test('Handles empty CSS', async () => {
    const root = await parse('');
    expect(root.type).toBe('root');
    expect(root.nodes.length).toBe(0);
  });

  test('Handles only whitespace', async () => {
    const root = await parse('   \n\n   ');
    expect(root.type).toBe('root');
    expect(root.nodes.length).toBe(0);
  });

  test('Handles only comments', async () => {
    const css = '/* comment */';
    const root = await parse(css);
    expect(root.nodes.length).toBe(1);
    expect(root.first?.type).toBe('comment');
  });

  test('Handles trailing semicolon after rule', async () => {
    const css = '.test { color: red; };';
    const root = await parse(css);
    const rule = root.first as any;

    // Should have ownSemicolon in raws
    expect(rule?.raws?.ownSemicolon).toBe(';');
  });

  test('Handles trailing semicolon with space', async () => {
    const css = '.test { color: red; } ;';
    const root = await parse(css);
    const rule = root.first as any;

    expect(rule?.raws?.ownSemicolon).toBe(' ;');
  });

  test('Handles nested structures', async () => {
    const css = '@media screen { .test { color: red; } }';
    const root = await parse(css);

    const mediaRule = root.first as any;
    expect(mediaRule.type).toBe('atrule');
    expect(mediaRule.nodes.length).toBe(1);

    const nestedRule = mediaRule.nodes[0];
    expect(nestedRule.type).toBe('rule');
    expect(nestedRule.selector).toBe('.test');
  });

  test('Handles at-rule without block', async () => {
    const css = '@charset "UTF-8";';
    const root = await parse(css);

    const atRule = root.first as any;
    expect(atRule.type).toBe('atrule');
    expect(atRule.name).toBe('charset');
    expect(atRule.params).toBe('"UTF-8"');
    // Statement at-rules should not have nodes field in JSON
    const json = atRule.toJSON();
    expect(json.nodes).toBeUndefined();
  });

  test('Handles at-rule with empty block', async () => {
    const css = '@page { }';
    const root = await parse(css);

    const atRule = root.first as any;
    expect(atRule.type).toBe('atrule');
    expect(atRule.name).toBe('page');
    // Block at-rules should have nodes property even when empty (PostCSS behavior)
    expect(atRule.nodes).toEqual([]);
  });

  test('Preserves exact source positions', async () => {
    const css = '.test { color: red; }';
    const root = await parse(css);

    const rule = root.first as any;
    expect(rule.source?.start).toBeDefined();
    expect(rule.source?.end).toBeDefined();
    expect(rule.source?.start?.line).toBe(1);
    expect(rule.source?.start?.column).toBe(1);
  });
});

describe('Real-world CSS patterns', () => {
  test('Handles complex selectors', async () => {
    const css = '.a > .b + .c:hover::before { color: red; }';
    const root = await parse(css);
    const rule = root.first as any;
    expect(rule.selector).toBe('.a > .b + .c:hover::before');
  });

  test('Handles vendor prefixes', async () => {
    const css = '@-webkit-keyframes slide { from { left: 0; } }';
    const root = await parse(css);
    const atRule = root.first as any;
    expect(atRule.name).toBe('-webkit-keyframes');
  });

  test('Handles calc() with spaces', async () => {
    const css = '.test { width: calc(100% - 20px); }';
    const root = await parse(css);
    const decl = (root.first as any)?.first;
    expect(decl.value).toBe('calc(100% - 20px)');
  });

  test('Handles CSS variables', async () => {
    const css = ':root { --main-color: #06c; }';
    const root = await parse(css);
    const decl = (root.first as any)?.first;
    expect(decl.prop).toBe('--main-color');
    expect(decl.value).toBe('#06c');
  });

  test('Handles multiple declarations', async () => {
    const css = '.test { margin: 0; padding: 0; border: 1px solid red; }';
    const root = await parse(css);
    const rule = root.first as any;
    expect(rule.nodes.length).toBe(3);
  });

  test('Handles comments between declarations', async () => {
    const css = '.test { color: red; /* comment */ background: blue; }';
    const root = await parse(css);
    const rule = root.first as any;
    expect(rule.nodes.length).toBe(3); // decl, comment, decl
    expect(rule.nodes[1].type).toBe('comment');
  });
});
