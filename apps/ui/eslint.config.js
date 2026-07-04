import { defineConfig, globalIgnores } from 'eslint/config'
import preferArrow from 'eslint-plugin-prefer-arrow'
import tseslint from 'typescript-eslint'

export default defineConfig([
  globalIgnores([
    'dist',
    'build',
    'src-tauri',
    'tests/tauri-fixture.{js,ts}',
    'src/routeTree.gen.ts',
    '.turbo',
    '.vite',
    '.tanstack',
  ]),
  {
    files: ['**/*.{ts,tsx}'],
    // base = parser + @typescript-eslint plugin, zero rules enabled.
    // Biome owns everything syntactic; ESLint only does type-aware work.
    extends: [tseslint.configs.base],
    plugins: {
      'prefer-arrow': preferArrow,
    },
    languageOptions: {
      parserOptions: {
        projectService: {
          allowDefaultProject: ['*.js'],
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      // Not in Biome: forces declarations -> arrow functions
      'prefer-arrow/prefer-arrow-functions': [
        'error',
        {
          disallowPrototype: true,
          singleReturnOnly: false,
          classPropertiesAllowed: false,
        },
      ],
      // Type-aware: require the real tsc checker, no Biome equivalent
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-unnecessary-condition': 'error',
      '@typescript-eslint/switch-exhaustiveness-check': 'error',
    },
  },
])
