import loguxTsConfig from '@logux/eslint-config/ts';
import prettierPlugin from 'eslint-plugin-prettier/recommended';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const dir = fileURLToPath(new URL('.', import.meta.url));

export default [
  ...loguxTsConfig,
  prettierPlugin,
  {
    files: ['**/*.ts'],
    languageOptions: {
      parserOptions: {
        project: resolve(dir, './tsconfig.eslint.json'),
      },
    },
    rules: {
      // PostCSS internal node construction genuinely requires any
      '@typescript-eslint/no-explicit-any': 'off',
      // Native addon loading via require() is unavoidable with NAPI
      '@typescript-eslint/no-var-requires': 'off',
      // CJS TypeScript projects don't use file extensions in imports
      'import/extensions': 'off',
      'n/global-require': 'off',
      // const is clearer than let for non-reassigned bindings
      'prefer-let/prefer-let': 'off',
    },
  },
  {
    files: ['benches/**/*.ts'],
    rules: {
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/no-unused-expressions': 'off',
      'no-console': 'off',
    },
  },
];
