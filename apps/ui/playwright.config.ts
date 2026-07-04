import { defineConfig } from '@playwright/test'

const isCI = !!process.env.CI

const TIMEOUT = 10_000

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  workers: 1,
  reporter: isCI ? [['github'], ['html', { open: 'never' }]] : 'list',
  timeout: 60_000, // to compensate for first cold boot
  expect: {
    timeout: TIMEOUT,
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.0,
      animations: 'disabled',
    },
  },
  use: {
    actionTimeout: TIMEOUT,
    trace: 'on-first-retry',
    video: 'on-first-retry',
    viewport: { width: 620, height: 610 },
  },
})
