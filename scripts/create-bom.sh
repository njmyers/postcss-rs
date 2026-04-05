#!/bin/bash

printf '\xEF\xBB\xBFa{}' >packages/parser/node_modules/postcss-parser-tests/cases/bom.css
printf '\xEF\xBB\xBFa{}' >packages/tokenizer/node_modules/postcss-parser-tests/cases/bom.css
printf '\xEF\xBB\xBFa{}' >packages/node-postcss/node_modules/postcss-parser-tests/cases/bom.css
