import { spawn } from 'node:child_process'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { test as base, expect, type Page } from '@playwright/test'
import kill from 'tree-kill'

declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>
      transformCallback?: (callback: (payload: unknown) => void, once?: boolean) => number
      [key: string]: unknown
    }
    __TAURI_EVENT_PLUGIN_INTERNALS__?: {
      unregisterListener: (event: string, eventId: number) => void
    }
    emitTauriEvent?: (event: string, payload: unknown) => void
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

const DEFAULT_CONFIG_STUB = {
  port: 8888,
  bypassDomains: [],
  proxyProtocol: 'http',
  proxyHost: '',
  proxyPort: 8080,
  pacServerPort: 8000,
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
      // Enough of the Tauri event plugin for `listen` to work: the real backend
      // hands the frontend a callback id, then invokes it by id when it emits.
      const callbacks = new Map<number, (payload: unknown) => void>()
      const listeners = new Map<string, Map<number, number>>()
      let nextId = 0

      window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ ?? {}

      window.__TAURI_INTERNALS__.transformCallback = (callback) => {
        const callbackId = ++nextId
        callbacks.set(callbackId, callback)
        return callbackId
      }

      window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
        if (cmd === 'load_config_command') return config
        if (cmd === 'is_server_running_command') return false
        if (cmd === 'plugin:event|listen') {
          const event = String(args?.event)
          const eventId = ++nextId
          const forEvent = listeners.get(event) ?? new Map<number, number>()
          forEvent.set(eventId, Number(args?.handler))
          listeners.set(event, forEvent)
          return eventId
        }
        return null
      }

      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener: (event, eventId) => {
          listeners.get(event)?.delete(eventId)
        },
      }

      window.emitTauriEvent = (event, payload) => {
        for (const [eventId, callbackId] of listeners.get(event) ?? []) {
          callbacks.get(callbackId)?.({ event, id: eventId, payload })
        }
      }
    }, DEFAULT_CONFIG_STUB)
    await use(page)
  },

  tauriProcess: [
    // biome-ignore lint/correctness/noEmptyPattern: Playwright requires the first fixture arg to be an object-destructuring pattern, even when no fixtures are used
    async ({}, use) => {
      const uiPath = path.resolve(__dirname, '..')

      const proc = spawn('pnpm', ['dev:vite'], {
        cwd: uiPath,
        shell: os.platform() === 'win32',
        detached: true,
        stdio: 'pipe',
      })

      await use(proc)

      await new Promise<void>((resolve) => {
        if (proc.pid) kill(proc.pid, 'SIGKILL', () => resolve())
        else resolve()
      })
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
