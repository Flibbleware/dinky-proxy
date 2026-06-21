import { useEffect, useState, type PropsWithChildren } from 'react'
import { toast } from 'sonner'
import { isServerRunning, loadConfig, startServer, stopServer } from '@/commands'
import { isValidConfiguration } from '@/utils'
import type { ConfigurationValues } from '@/screens/configuration/types'
import { AppContext, type AppStatus } from './useInitialisation'

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

export const InitialisationProvider = ({ children }: PropsWithChildren) => {
  const [status, setStatus] = useState<AppStatus>('loading')
  const [config, setConfig] = useState<ConfigurationValues>(DEFAULT_CONFIG)
  const [isRunning, setIsRunning] = useState<boolean | null>(null)
  const [isTogglingServer, setIsTogglingServer] = useState(false)

  useEffect(() => {
    const controller = new AbortController()

    void (async () => {
      try {
        const [rawConfig, running] = await Promise.all([loadConfig(), isServerRunning()])
        if (controller.signal.aborted) return

        if (isValidConfiguration(rawConfig)) {
          setConfig(rawConfig)
        } else {
          toast.warning("Couldn't load your saved configuration — starting with defaults.")
        }

        setIsRunning(running)
        setStatus('ready')
      } catch {
        if (controller.signal.aborted) return
        setStatus('failed')
      }
    })()

    return () => controller.abort()
  }, [])

  const toggleServer = async () => {
    if (isRunning === null) return
    const stopping = isRunning
    setIsTogglingServer(true)
    try {
      if (stopping) {
        await stopServer()
        setIsRunning(false)
        toast.success('Server stopped')
        return
      }

      await startServer()
      setIsRunning(true)
      toast.success('Server started')
    } catch (error) {
      console.error('Failed to toggle server', error)
      toast.error(stopping ? 'Failed to stop server' : 'Failed to start server')
    } finally {
      setIsTogglingServer(false)
    }
  }

  return (
    <AppContext value={{ status, config, isRunning, isTogglingServer, setIsRunning, toggleServer }}>
      {children}
    </AppContext>
  )
}
