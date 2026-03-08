---
layout: home

hero:
  name: postcss-rs
  text: PostCSS parsing at Rust speed
  tagline: A drop-in replacement for the PostCSS parser — 3–5× faster, 100% compatible.
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
  - title: Blazing Fast
    details: Written in Rust, postcss-rs parses CSS 3–5× faster than the standard PostCSS parser — with near-zero overhead from the Rust/JS boundary.
  - title: Drop-in Compatible
    details: Returns real PostCSS Root instances. Works with all existing PostCSS plugins and tooling — no migration needed.
  - title: Zero Adapter Overhead
    details: Rust outputs PostCSS-compatible JSON directly. No intermediate transformation layer means no extra work at runtime.
---
