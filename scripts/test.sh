#!/bin/bash

pnpm build:debug
pnpm test
cargo test
