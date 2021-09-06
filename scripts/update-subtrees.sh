#!/bin/bash
git subtree pull \
  --prefix=tests \
  git@github.com:postcss/postcss-parser-tests.git \
  main \
  --squash
