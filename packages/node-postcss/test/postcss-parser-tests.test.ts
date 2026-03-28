import { eachTest, jsonify } from 'postcss-parser-tests';
import { Input } from 'postcss';
import type { Root } from 'postcss';
import {
  parseJsonAsync,
  parseJsonSync,
  parseOffsetsSync,
  parseOffsetsAsync,
} from '../src/index';

function assignInput(node: any, input: Input): void {
  if (node.source) {
    node.source.input = input;
  }
  if (node.nodes) {
    for (const child of node.nodes) {
      assignInput(child, input);
    }
  }
}

const parsers: Array<[string, (css: string) => Root | Promise<Root>]> = [
  ['parseJsonAsync',   (css) => parseJsonAsync(css)],
  ['parseJsonSync',    (css) => parseJsonSync(css)],
  ['parseOffsetsSync', (css) => parseOffsetsSync(css)],
  ['parseOffsetsAsync',(css) => parseOffsetsAsync(css)],
];

describe.each(parsers)('postcss-parser-tests (%s)', (_name, parse) => {
  eachTest((name, css, expectedJson) => {
    test(name, async () => {
      const root = await parse(css);
      assignInput(root, new Input(css));
      expect(jsonify(root)).toEqual(expectedJson);
    });
  });
});
