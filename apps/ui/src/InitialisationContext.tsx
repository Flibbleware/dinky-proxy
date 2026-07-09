import { type PropsWithChildren, useEffect, useState } from 'react'
import { toast } from 'sonner'
import {
  type ConfigurationValues,
  isServerRunning,
  loadConfig,
  startServer,
  stopServer,
} from '@/commands'
import { AppContext, type LoadState } from './useInitialisation'

const INITIAL_STATE: LoadState = { status: 'loading', config: null, isRunning: null }

const load = async (setState: (state: LoadState) => void, signal?: AbortSignal) => {
  try {
    const [config, isRunning] = await Promise.all([loadConfig(), isServerRunning()])
    if (signal?.aborted) return
    setState({ status: 'ready', config, isRunning })
  } catch (error) {
    if (signal?.aborted) return
    console.error('Failed to load configuration', error)
    setState({ status: 'failed', config: null, isRunning: null })
  }
}

export const InitialisationProvider = ({ children }: PropsWithChildren) => {
  const [state, setState] = useState<LoadState>(INITIAL_STATE)
  const [isTogglingServer, setIsTogglingServer] = useState(false)

  useEffect(() => {
    const controller = new AbortController()
    void load(setState, controller.signal)
    return () => controller.abort()
  }, [])

  const retryInitialisation = () => {
    setState(INITIAL_STATE)
    void load(setState)
  }

  const setIsRunning = (value: boolean) =>
    setState((prev) => (prev.status === 'ready' ? { ...prev, isRunning: value } : prev))

  const setConfig = (config: ConfigurationValues) =>
    setState((prev) => (prev.status === 'ready' ? { ...prev, config } : prev))

  const toggleServer = async () => {
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
  }

  return (
    <AppContext
      value={{
        state,
        isTogglingServer,
        setIsRunning,
        setConfig,
        retryInitialisation,
        toggleServer,
      }}
    >
      {children}
    </AppContext>
  )
}
