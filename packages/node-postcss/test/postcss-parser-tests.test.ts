import { eachTest, jsonify } from 'postcss-parser-tests';
import { Input } from 'postcss';
import { parse } from '../src/index';

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

describe('postcss-parser-tests', () => {
  eachTest((name, css, expectedJson) => {
    test(name, async () => {
      const root = await parse(css);
      assignInput(root, new Input(css));
      expect(jsonify(root)).toEqual(expectedJson);
    });
  });
});
