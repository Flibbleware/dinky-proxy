import { test as base, expect, type Page } from '@playwright/test'
import { spawn } from 'child_process'
import path from 'path'
import { fileURLToPath } from 'url'
import kill from 'tree-kill'
import os from 'os'

declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>
      [key: string]: unknown
    }
  }
}

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const waitForVite = async (url: string, timeout = 45000) => {
  const start = Date.now()

  while (Date.now() - start < timeout) {
    try {
      const res = await fetch(url)
      if (res.ok) return
    } catch {
      await new Promise((r) => setTimeout(r, 300))
    }
  }

  throw new Error('Vite server did not start in time')
}

const killLeftoverTauri = async () => {
  const processName = os.platform() === 'win32' ? 'dinkyproxy-ui.exe' : 'dinkyproxy-ui'

  const cmd =
    os.platform() === 'win32' ? `taskkill /F /IM ${processName}` : `pkill -f ${processName}`

  await new Promise((resolve) => {
    spawn(cmd, { shell: true }).on('close', resolve)
  })
}

const DEFAULT_CONFIG_STUB = {
  port: 8888,
  bypassDomains: [],
  proxyProtocol: 'http',
  proxyHost: '',
  proxyPort: 8080,
  pacServerPort: 8000,
  networkTarget: 'Wi-Fi',
  username: '',
  password: '',
}

export const test = base.extend<
  { page: Page },
  {
    tauriProcess: ReturnType<typeof spawn>
    pageUrl: string
  }
>({
  page: async ({ page }, use) => {
    await page.addInitScript((config) => {
      window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ ?? {}
      window.__TAURI_INTERNALS__.invoke = async (cmd) => {
        if (cmd === 'load_config_command') return config
        if (cmd === 'is_server_running_command') return false
        return null
      }
    }, DEFAULT_CONFIG_STUB)
    await use(page)
  },

  tauriProcess: [
    async ({}, use) => {
      const uiPath = path.resolve(__dirname, '..')

      const proc = spawn('pnpm', ['tauri', 'dev'], {
        cwd: uiPath,
        env: {
          ...process.env,
          TAURI_DEV_WATCHER: 'false',
        },
        shell: os.platform() === 'win32',
        detached: true,
        stdio: 'pipe',
      })

      await use(proc)

      if (proc.pid) kill(proc.pid, 'SIGKILL')

      await killLeftoverTauri()
    },
    { scope: 'worker' },
  ],

  pageUrl: [
    async ({ tauriProcess: _tauriProcess }, use) => {
      const url = 'http://localhost:5173'

      await waitForVite(url)
      await use(url)
    },
    { scope: 'worker' },
  ],
})

export { expect }

export const fullPageScreenshot = (page: Page, name: string) =>
  expect(page).toHaveScreenshot(`${name}.png`, { fullPage: true })
