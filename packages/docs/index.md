---
layout: home

hero:
  name: postcss-rs
  text: PostCSS parsing at Rust speed
  tagline: A drop-in replacement for the PostCSS parser — 100% compatible, 11–36% faster than postcss.
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Why postcss-rs?
      link: /guide/why
    - theme: alt
      text: View on GitHub
      link: https://github.com/njmyers/postcss-rs

features:
  - title: 11–36% Faster than PostCSS
    details: Written in Rust, postcss-rs outperforms the standard PostCSS parser by 11% on small, 17% on medium, and 36% on large CSS files. The single-pass offsets sync mode eliminates the intermediate AST allocation for maximum throughput.
  - title: Drop-in Compatible
    details: Returns real PostCSS Root instances. Works with all existing PostCSS plugins and tooling — no migration needed.
  - title: Zero Adapter Overhead
    details: Rust outputs PostCSS-compatible JSON directly. No intermediate transformation layer means no extra work at runtime.
---
