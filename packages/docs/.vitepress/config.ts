import { defineConfig } from 'vitepress';

export default defineConfig({
  description:
    'A high-performance PostCSS-compatible CSS parser powered by Rust',
  themeConfig: {
    footer: {
      copyright: 'Copyright © 2024 Nick Myers',
      message: 'Released under the MIT License.',
    },

    nav: [
      { link: '/guide/getting-started', text: 'Guide' },
      { link: '/api/', text: 'API' },
      {
        link: '/rustdoc/postcss_rs_parser/index.html',
        target: '_blank',
        text: 'Rust API',
      },
      { link: 'https://github.com/njmyers/postcss-rs', text: 'GitHub' },
    ],

    sidebar: [
      {
        items: [
          { link: '/guide/why', text: 'Why postcss-rs?' },
          { link: '/guide/getting-started', text: 'Getting Started' },
          { link: '/guide/architecture', text: 'Architecture' },
        ],
        text: 'Guide',
      },
      {
        items: [{ link: '/api/', text: 'Overview' }],
        text: 'API Reference',
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/njmyers/postcss-rs' },
    ],
  },

  title: 'postcss-rs',
});
