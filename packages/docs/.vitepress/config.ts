import { defineConfig } from 'vitepress'
import parserSidebar from '../rust-api/parser-sidebar.json'
import tokenizerSidebar from '../rust-api/tokenizer-sidebar.json'

export default defineConfig({
  title: 'postcss-rs',
  description: 'A high-performance PostCSS-compatible CSS parser powered by Rust',

  themeConfig: {
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'API', link: '/api/' },
      { text: 'Rust API', link: '/rust-api/parser/' },
      { text: 'GitHub', link: 'https://github.com/njmyers/postcss-rs' }
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Guide',
          items: [
            { text: 'Why postcss-rs?', link: '/guide/why' },
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Architecture', link: '/guide/architecture' }
          ]
        }
      ],
      '/api/': [
        {
          text: 'API Reference',
          items: [
            { text: 'Overview', link: '/api/' }
          ]
        }
      ],
      '/rust-api/': [
        {
          text: 'Rust API',
          items: [
            parserSidebar,
            tokenizerSidebar
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/njmyers/postcss-rs' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024 Nick Myers'
    }
  }
})
