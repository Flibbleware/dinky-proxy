const path = require('node:path')

const UI_DIR = path.join(__dirname, 'apps/ui')

// Pre-commit checks run against staged files only (see .husky/pre-commit).
// `biome check --write` formats, applies safe lint fixes, and organizes imports;
// it exits non-zero on unfixable lint errors, which blocks the commit.
// `tsc` is not run here — it needs the whole program, so it stays in CI
// (see .github/workflows/frontend.yml).
module.exports = {
  // Biome must run with its cwd inside apps/ui: invoked from the repo root it
  // treats the root as an implicit project root and rejects apps/ui/biome.json
  // as a "nested root" config. `pnpm --filter` runs the binary in the package
  // dir, so paths are made relative to apps/ui.
  'apps/ui/**/*.{ts,tsx,js,json,jsonc,css}': (files) => {
    const rel = files.map((f) => `'${path.relative(UI_DIR, f)}'`).join(' ')
    return `pnpm --filter dinkyproxy-ui exec biome check --write --no-errors-on-unmatched ${rel}`
  },
  // Format the whole crate whenever any Rust file is staged. Using a function
  // keeps lint-staged from appending file paths to `cargo fmt`.
  'apps/ui/src-tauri/**/*.rs': () =>
    'cargo fmt --manifest-path apps/ui/src-tauri/Cargo.toml',
}
