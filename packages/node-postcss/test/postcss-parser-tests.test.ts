import { Input } from 'postcss';
import type { Root } from 'postcss';
import { eachTest, jsonify } from 'postcss-parser-tests';
import { describe, expect, test } from 'vitest';

import {
  parseJsonAsync,
  parseJsonSync,
  parseOffsetsAsync,
  parseOffsetsSync,
} from '../src/index';

function assignInput(node: any, input: Input): void {
  if (node.source) {
    node.source.input = input;
  }
  if (node.nodes) {
    for (let child of node.nodes) {
      assignInput(child, input);
    }
  }
}

const parsers: [string, (css: string) => Promise<Root> | Root][] = [
  ['parseJsonAsync', css => parseJsonAsync(css)],
  ['parseJsonSync', css => parseJsonSync(css)],
  ['parseOffsetsSync', css => parseOffsetsSync(css)],
  ['parseOffsetsAsync', css => parseOffsetsAsync(css)],
];

describe.each(parsers)('postcss-parser-tests (%s)', (_name, parse) => {
  eachTest((name: string, css: string, expectedJson: string) => {
    test(name, async () => {
      let root = await parse(css);
      assignInput(root, new Input(css));
      expect(jsonify(root)).toEqual(expectedJson);
    });
  });
});
