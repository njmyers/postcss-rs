import loguxTsConfig from '@logux/eslint-config/ts'
import prettierPlugin from 'eslint-plugin-prettier/recommended'

export default [
  ...loguxTsConfig,
  prettierPlugin,
  {
    rules: {
      'prefer-let/prefer-let': 'off'
    }
  },
  {
    files: ['test/update.js'],
    rules: {
      'n/no-missing-require': 'off'
    }
  }
]
