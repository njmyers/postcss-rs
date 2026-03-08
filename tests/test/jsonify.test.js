let postcss = require('postcss')
let { test } = require('uvu')
let { equal } = require('uvu/assert')

let jsonify = require('../jsonify')

test('converts to JSON string', () => {
  let node = postcss.rule()
  equal(jsonify(node), { nodes: [], raws: {}, type: 'rule' })
})

test('converts source.input', () => {
  let node = postcss.rule({
    source: {
      input: new postcss.Input('test', { from: '/a.css' })
    }
  })
  node.each(() => {})
  equal(jsonify(node), {
    nodes: [],
    raws: {},
    source: {},
    type: 'rule'
  })
})

test('converts source.input recursively', () => {
  let rule = postcss.rule({
    source: {
      end: {
        column: 15,
        line: 1,
        offset: 14
      },
      input: new postcss.Input('test', { from: '/a.css' }),
      start: {
        column: 1,
        line: 1,
        offset: 0
      }
    }
  })
  let root = postcss.root()
  root.append(rule)

  equal(
    jsonify(root),
    '{\n' +
      '  "raws": {},\n' +
      '  "type": "root",\n' +
      '  "nodes": [\n' +
      '    {\n' +
      '      "raws": {},\n' +
      '      "source": {\n' +
      '        "end": {\n' +
      '          "offset": 14,\n' +
      '          "line": 1,\n' +
      '          "column": 15\n' +
      '        },\n' +
      '        "start": {\n' +
      '          "offset": 0,\n' +
      '          "line": 1,\n' +
      '          "column": 1\n' +
      '        }\n' +
      '      },\n' +
      '      "type": "rule",\n' +
      '      "nodes": []\n' +
      '    }\n' +
      '  ]\n' +
      '}'
  )
})

test.run()
