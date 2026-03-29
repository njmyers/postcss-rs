---
layout: home

hero:
  name: postcss-rs
  text: PostCSS parsing at Rust speed
  tagline: A drop-in replacement for the PostCSS parser — 100% compatible, ~15% faster than postcss on large CSS.
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
  - title: Faster than PostCSS on Large CSS
    details: Written in Rust, postcss-rs outperforms the standard PostCSS parser by ~15% on large CSS files. The offsets sync mode is the fastest path and continues to improve.
  - title: Drop-in Compatible
    details: Returns real PostCSS Root instances. Works with all existing PostCSS plugins and tooling — no migration needed.
  - title: Zero Adapter Overhead
    details: Rust outputs PostCSS-compatible JSON directly. No intermediate transformation layer means no extra work at runtime.
---
