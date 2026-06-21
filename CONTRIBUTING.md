# Contributing

## Monorepo layout

- Root: workspace manifests (`pnpm-workspace.yaml`, `turbo.json`), lockfile, and shared dotfiles.
- `apps/ui`: Tauri application (Vite/React frontend + embedded Rust server code in `src-tauri/`).
- `scripts/`: `release.sh` (version bump + tag) and `collect-builds.js` (copies build output into `dist/`).
- Root also holds `proxy.pac`.

### Key concepts

- **pnpm workspace**: one lockfile (`pnpm-lock.yaml`) and shared `node_modules`. Run `pnpm install` at the root.
- **Tauri**: Rust backend and native shell wrapper. `pnpm dev` launches `tauri dev`, which boots Vite and the Rust process together.
- **Turborepo**: orchestrates shared scripts such as `pnpm lint`. All day‑to‑day commands can be run from the repo root using `pnpm --filter`.

## Prerequisites — macOS

- macOS with Homebrew
- Rust toolchain (`brew install rust` or `rustup`)
- Node.js LTS (22+) and pnpm 11

## Prerequisites — Windows

- Windows with winget
- Rust toolchain (`winget install --id=Rustlang.Rustup -e`)
- NVM (`winget install -e --id CoreyButler.NVMforWindows`)
- Node.js (`nvm install lts; nvm use lts;`)
- pnpm (`winget install --id=pnpm.pnpm -e`)
- MSVC v143 Build Tools: Windows 10/11 SDK, MSVC linker (link.exe), C++ libraries

## Recommended editor setup

- VSCode rust-analyzer extension
- VSCode Rust Syntax extension
- VSCode Biome extension (formatting + linting)
- VSCode ESLint extension (optional — surfaces the type-aware rules Biome can't run)

## Getting started

Clone the repo and install dependencies:

```bash
git clone <repo-url>
pnpm install
pnpm dev  # also generates routeTree.gen.ts, which fixes TypeScript errors on first run
```

## Commands

### Run dev app

```bash
pnpm dev
```

`tauri dev` automatically starts the Vite dev server on :5173 and the Rust backend inside the same process. Logs from both land in the same terminal.

### Run React dev app only (Vite, no Tauri)

```bash
pnpm --filter ./apps/ui dev:vite
```

### Formatting and linting

```bash
pnpm --filter ./apps/ui format  # biome: format + safe lint fixes + import sort (UI package only)
pnpm lint                       # biome check + eslint type-aware rules (Rust clippy/rustfmt run in CI)
```

A Husky `pre-commit` hook runs [lint-staged](lint-staged.config.cjs) on staged files: `biome check --write` for `apps/ui` TS/CSS/JSON and `cargo fmt` for staged Rust. Commits are auto-formatted and blocked on unfixable Biome lint errors. The hooks install automatically via the `prepare` script on `pnpm install`.

### Dead code detection

```bash
pnpm --filter ./apps/ui dead-code
```

Runs [Fallow](https://docs.fallow.tools) to detect unused exports, unreachable files, and unlisted dependencies across the React app. The same check runs in the Frontend CI workflow.

### Build

```bash
pnpm build
```

Runs in order:

1. `pnpm clean`
2. `pnpm --filter ./apps/ui build` (Vite/React assets)
3. `pnpm --filter ./apps/ui tauri:build` (native installers/binaries)
4. Copies `apps/ui/dist` → `dist/web` and `apps/ui/src-tauri/target/release/bundle` → `dist/tauri`

Grab the files you need from `dist/` for distribution.

### Unit tests

```bash
pnpm test:unit
```

Rust unit tests covering the pure logic in the Tauri backend. No network or OS dependencies.

### E2E tests

```bash
pnpm test:e2e
```

Runs the Playwright suite from the repo root. Tests live in `apps/ui/tests/e2e/`. On failure, screenshots, videos, and traces are written to `apps/ui/playwright-report/`.

### Visual regression

Key tests capture full-page screenshots and compare them against committed baseline snapshots using Playwright's `toHaveScreenshot`.

To regenerate baselines locally:

```bash
pnpm test:e2e:update
```

Commit the updated snapshots alongside the code change — if this is missed the CI step will block the PR.

## Releasing

```bash
pnpm release <version>          # e.g. pnpm release 1.0.0
pnpm release <version> --push   # bump, tag, and push to origin in one step
```

This script:
1. Validates the working tree is clean and the tag doesn't already exist.
2. Updates the version in `apps/ui/src-tauri/Cargo.toml` and syncs `Cargo.lock`.
3. Creates a `chore: release vX.Y.Z` commit and a `vX.Y.Z` tag.
4. Optionally pushes the commit and tag to origin (omit `--push` to inspect before pushing).

Pushing the tag triggers the **Build** CI workflow, which builds the native installers and publishes them to a GitHub Release.

## CI (GitHub Actions)

Four workflows run automatically:

| Workflow     | Trigger                                            | What it does                                                                                                          |
| ------------ | -------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| **Frontend** | Every PR and push to `main`                        | Runs Biome (format + lint) and ESLint type-aware rules, TypeScript type-check, and Fallow dead code detection (`ubuntu-latest`) |
| **Backend**  | Every PR or push to `main` touching `src-tauri/**` | Runs Rust unit tests, Clippy lint, and rustfmt format check (`ubuntu-latest`)                                         |
| **Tests**    | Every PR and push to `main`                        | Runs Playwright E2E tests (`macos-latest`)                                                                            |
| **Build**    | Push of a `v*` tag (e.g. `v0.9.3`)                 | Verifies the tag matches the `Cargo.toml` version, builds native installers on macOS (`.dmg`) and Windows (`.msi` / `.exe`), and publishes them to a GitHub Release |

Playwright screenshots, videos, and traces are uploaded as an artifact (`playwright-artifacts`) on every run, including failures.
