#!/usr/bin/env node

/**
 * Test to compare JSON formats between postcss-rs and PostCSS
 */

const postcss = require('postcss');

// Sample CSS
const css = `.test { color: red; }`;

// Parse with PostCSS
const root = postcss.parse(css, { from: 'test.css' });

// Serialize to JSON
const postcssJSON = root.toJSON();

console.log('PostCSS JSON format:');
console.log(JSON.stringify(postcssJSON, null, 2));

console.log('\n\nExpected structure for fromJSON:');
console.log('- type: string (root, rule, atrule, decl, comment)');
console.log('- For Rule: selector, nodes[]');
console.log('- For AtRule: name, params, nodes[]');
console.log('- For Declaration: prop, value, important');
console.log('- For Comment: text');
console.log('- raws: object with formatting info');
console.log('- source: { start: {line, column, offset?}, end, input }');

// Test fromJSON
console.log('\n\nTesting fromJSON:');
const restored = postcss.fromJSON(postcssJSON);
console.log('Successfully restored:', restored.toString());
console.log('Matches original:', restored.toString() === root.toString());
