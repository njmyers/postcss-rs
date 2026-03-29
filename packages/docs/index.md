---
layout: home

hero:
  name: postcss-rs
  text: PostCSS parsing at Rust speed
  tagline: A drop-in replacement for the PostCSS parser — 100% compatible, performance on par with postcss.
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
  - title: Performance on Par with PostCSS
    details: Written in Rust, postcss-rs currently performs on par with the standard PostCSS parser. The offsets sync mode is the fastest path — significant improvements are in progress.
  - title: Drop-in Compatible
    details: Returns real PostCSS Root instances. Works with all existing PostCSS plugins and tooling — no migration needed.
  - title: Zero Adapter Overhead
    details: Rust outputs PostCSS-compatible JSON directly. No intermediate transformation layer means no extra work at runtime.
---
