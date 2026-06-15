# DinkyProxy

Lightweight proxy helper built as a single Tauri application (React + TanStack Router on the frontend, Rust backend commands) inside a pnpm/Turborepo workspace.

## Monorepo layout

- Root: workspace manifests (`pnpm-workspace.yaml`, `turbo.json`), lockfile, and shared dotfiles.
- `apps/ui`: Tauri application (Vite/React frontend + embedded Rust server code in `src-tauri/`).
- Root also holds `proxy.pac`.

### Key concepts

- **pnpm workspace**: one lockfile (`pnpm-lock.yaml`) and shared `node_modules`. Run `pnpm install` at the root.
- **Tauri**: Rust backend and native shell wrapper. `pnpm dev` launches `tauri dev`, which boots Vite and the Rust process together.
- **Turborepo**: still orchestrates shared scripts such as `pnpm lint`, but most day‑to‑day commands run from `apps/ui`.

## Prerequisites - Mac

- macOS with Homebrew
- Rust toolchain (`brew install rust` or `rustup`)
- Node.js LTS (22+) and pnpm 11

## Prerequisites - Windows

- Windows with winget
- Rust toolchain (`winget install --id=Rustlang.Rustup -e`)
- NVM (`winget install -e --id CoreyButler.NVMforWindows`)
- Node.js (`nvm install lts; nvm use lts;`)
- pnpm (`winget install --id=pnpm.pnpm -e`)
- MSVC v143 Build Tools: Windows 10/11 SDK, MSVC linker (link.exe), C++ libraries

## Prerequisites - Developer Experience

- VSCode rust-analyser extension
- VSCode Rust Syntax extension
- VSCode Prettier extension
- VSCode ESLint extension

Clone the repo and set your working directory:

```bash
git clone <repo-url>
cd dinky-proxy\
pnpm dev # generate routeTree.gen.ts to fix TypeScript errors
```

## Install dependencies (root)

```bash
pnpm install
```

This installs `turbo` and wires workspace packages.

## Dev workflow

```bash
pnpm dev
pnpm lint
```

`tauri dev` automatically starts the Vite dev server on :5173 and the Rust backend inside the same process. Logs from both land in the same terminal.

### Run React app only

```bash
cd apps/ui
pnpm dev
```

### Formatting and linting

```bash
cd apps/ui && pnpm format  # prettier (UI package only)
pnpm lint                  # eslint across the workspace (Rust clippy/rustfmt run in CI)
```

### Building

```bash
pnpm build
```

`pnpm build` runs, in order:

1. `pnpm clean`
2. `pnpm --filter ./apps/ui build` (Vite/React assets)
3. `pnpm --filter ./apps/ui tauri:build` (native installers/binaries)
4. Copies `apps/ui/dist` → `dist/web` and `apps/ui/src-tauri/target/release/bundle` → `dist/tauri`

Grab the files you need from `dist/` (web assets or OS-specific bundles) for distribution.

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

## CI (GitHub Actions)

Four workflows run automatically:

| Workflow     | Trigger                                            | What it does                                                                                                          |
| ------------ | -------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| **Frontend** | Every PR                                           | Runs ESLint, TypeScript type-check, and Prettier format check (`ubuntu-latest`)                                       |
| **Backend**  | Every PR or push to `main` touching `src-tauri/**` | Runs Rust unit tests, Clippy lint, and rustfmt format check (`ubuntu-latest`)                                         |
| **Tests**    | Every PR and push to `main`                        | Runs Playwright E2E tests (`macos-latest`)                                                                            |
| **Build**    | Push to `main`                                     | Builds native installers on macOS (`.dmg`) and Windows (`.msi` / `.exe`) and uploads them as GitHub Actions artifacts |

Playwright screenshots, videos, and traces are uploaded as an artifact (`playwright-artifacts`) on every run, including failures.
