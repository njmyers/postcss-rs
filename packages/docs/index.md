---
layout: home

hero:
  name: postcss-rs
  text: PostCSS parsing at Rust speed
  tagline: A drop-in replacement for the PostCSS parser — 100% compatible, faster for large CSS files.
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
  - title: Faster for Large Files
    details: Written in Rust, postcss-rs parses large CSS files (~30 KB+) up to ~12% faster than the standard PostCSS parser using the offsets sync mode. Performance scales with file size.
  - title: Drop-in Compatible
    details: Returns real PostCSS Root instances. Works with all existing PostCSS plugins and tooling — no migration needed.
  - title: Zero Adapter Overhead
    details: Rust outputs PostCSS-compatible JSON directly. No intermediate transformation layer means no extra work at runtime.
---
