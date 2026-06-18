import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { createFileRoute } from '@tanstack/react-router'
import Configuration from '../screens/configuration'
import type { ConfigurationFormValues } from '@/screens/configuration/types'

const loadConfig = async (): Promise<ConfigurationFormValues | null> => {
  try {
    const config = (await invoke('load_config_command')) as ConfigurationFormValues

    if (
      typeof config.port === 'number' &&
      Array.isArray(config.bypassDomains) &&
      typeof config.proxyHost === 'string' &&
      typeof config.proxyPort === 'number' &&
      typeof config.pacServerPort === 'number' &&
      typeof config.networkTarget === 'string' &&
      typeof config.username === 'string' &&
      typeof config.password === 'string'
    ) {
      return config satisfies ConfigurationFormValues
    }

    return null
  } catch (error) {
    console.error('Failed to load config', error)
    return null
  }
}

const Landing = () => {
  const [initialConfig, setInitialConfig] = useState<ConfigurationFormValues | null>()

  useEffect(() => {
    const controller = new AbortController()

    void (async () => {
      const result = await loadConfig()
      if (!controller.signal.aborted) setInitialConfig(result)
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

        {initialConfig !== undefined && <Configuration initialValues={initialConfig ?? {}} />}
      </div>
    </div>
  )
}

export const Route = createFileRoute('/landing')({
  component: Landing,
})
