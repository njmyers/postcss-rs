import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'postcss-rs',
  description:
    'A high-performance PostCSS-compatible CSS parser powered by Rust',

  themeConfig: {
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'API', link: '/api/' },
      {
        text: 'Rust API',
        link: '/rustdoc/postcss_rs_parser/index.html',
        target: '_blank',
      },
      { text: 'GitHub', link: 'https://github.com/njmyers/postcss-rs' },
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Why postcss-rs?', link: '/guide/why' },
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Architecture', link: '/guide/architecture' },
        ],
      },
      {
        text: 'API Reference',
        items: [{ text: 'Overview', link: '/api/' }],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/njmyers/postcss-rs' },
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024 Nick Myers',
    },
  },
});
