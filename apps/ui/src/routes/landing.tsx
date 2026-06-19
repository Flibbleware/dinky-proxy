import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { createFileRoute } from '@tanstack/react-router'
import { toast } from 'sonner'
import Configuration from '../screens/configuration'
import type { ConfigurationValues } from '@/screens/configuration/types'
import { isValidConfiguration } from '@/utils'

const DEFAULT_CONFIG: ConfigurationValues = {
  port: 8888,
  bypassDomains: ['imgur.com'],
  proxyProtocol: 'http',
  proxyHost: '',
  proxyPort: 8080,
  pacServerPort: 8000,
  networkTarget: 'Wi-Fi',
  username: '',
  password: '',
}

type LoadResult = { config: ConfigurationValues; recovered: boolean }

const loadConfig = async (): Promise<LoadResult> => {
  try {
    const config = await invoke('load_config_command')

    if (isValidConfiguration(config)) {
      return { config, recovered: true }
    }

    return { config: DEFAULT_CONFIG, recovered: false }
  } catch (error) {
    console.error('Failed to load config', error)
    return { config: DEFAULT_CONFIG, recovered: false }
  }
}

const Landing = () => {
  const [initialConfig, setInitialConfig] = useState<ConfigurationValues>()

  useEffect(() => {
    const controller = new AbortController()

    void (async () => {
      const { config, recovered } = await loadConfig()
      if (controller.signal.aborted) return

      setInitialConfig(config)
      if (!recovered) {
        toast.warning("Couldn't load your saved configuration — starting with defaults.")
      }
    })()

    return () => controller.abort()
  }, [])

  return (
    <div className="min-h-screen bg-slate-950 text-slate-50">
      <div className="mx-auto flex max-w-3xl flex-col gap-8 px-6 pt-7 pb-7">
        <header className="flex flex-col gap-4">
          <div>
            <p className="text-[11px] tracking-[0.35em] text-emerald-200/80 uppercase">
              DinkyProxy
            </p>
            <h1 className="mt-2 text-3xl font-semibold text-white md:text-4xl">Configuration</h1>
            <p className="mt-3 max-w-2xl text-sm text-slate-300">
              Configure the proxy settings and domains you would like to funnel through the proxy.
            </p>
          </div>
        </header>

        {initialConfig != null && <Configuration initialValues={initialConfig} />}
      </div>
    </div>
  )
}

export const Route = createFileRoute('/landing')({
  component: Landing,
})
