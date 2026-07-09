import { type PropsWithChildren, useCallback, useEffect, useMemo, useState } from 'react'
import { toast } from 'sonner'
import {
  type ConfigurationValues,
  isServerRunning,
  loadConfig,
  startServer,
  stopServer,
} from '@/commands'
import { AppContext, type AppContextValue, type LoadState } from './useInitialisation'

const INITIAL_STATE: LoadState = { status: 'loading', config: null, isRunning: null }

export const InitialisationProvider = ({ children }: PropsWithChildren) => {
  const [state, setState] = useState<LoadState>(INITIAL_STATE)
  const [isTogglingServer, setIsTogglingServer] = useState(false)

  const load = useCallback(async (signal?: AbortSignal) => {
    try {
      const [config, isRunning] = await Promise.all([loadConfig(), isServerRunning()])
      if (signal?.aborted) return
      setState({ status: 'ready', config, isRunning })
    } catch (error) {
      if (signal?.aborted) return
      console.error('Failed to load configuration', error)
      setState({ status: 'failed', config: null, isRunning: null })
    }
  }, [])

  useEffect(() => {
    const controller = new AbortController()
    void load(controller.signal)
    return () => controller.abort()
  }, [load])

  const retryInitialisation = useCallback(() => {
    setState(INITIAL_STATE)
    void load()
  }, [load])

  const setIsRunning = useCallback(
    (value: boolean) =>
      setState((prev) => (prev.status === 'ready' ? { ...prev, isRunning: value } : prev)),
    [],
  )

  const setConfig = useCallback(
    (config: ConfigurationValues) =>
      setState((prev) => (prev.status === 'ready' ? { ...prev, config } : prev)),
    [],
  )

  const toggleServer = useCallback(async () => {
    if (state.status !== 'ready') return
    const stopping = state.isRunning
    setIsTogglingServer(true)
    try {
      if (stopping) {
        await stopServer()
        setIsRunning(false)
        toast.success('Proxy Disabled')
        return
      }

      await startServer()
      setIsRunning(true)
      toast.success('Proxy Enabled')
    } catch (error) {
      console.error('Failed to toggle server', error)
      toast.error(stopping ? 'Failed to stop server' : 'Failed to start server')
    } finally {
      setIsTogglingServer(false)
    }
  }, [state, setIsRunning])

  const value = useMemo<AppContextValue>(
    () => ({ state, isTogglingServer, setIsRunning, setConfig, retryInitialisation, toggleServer }),
    [state, isTogglingServer, setIsRunning, setConfig, retryInitialisation, toggleServer],
  )

  return <AppContext value={value}>{children}</AppContext>
}
