#!/usr/bin/env node

const { promises: fs } = require('fs');
const path = require('path');
const { postcss } = require('../');

const CASES = path.resolve(__dirname, '../../../tests/cases');

(async () => {
  const css = await fs.readFile(path.resolve(CASES, 'decls.css'), 'utf-8');
  await postcss.parse(css);
})();
