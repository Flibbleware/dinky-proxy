import { type PropsWithChildren, useEffect, useState } from 'react'
import { toast } from 'sonner'
import { isServerRunning, loadConfig, startServer, stopServer } from '@/commands'
import { AppContext, type AppContextValue } from './useInitialisation'

type LoadState = Pick<AppContextValue, 'status' | 'config' | 'isRunning'>

const INITIAL_STATE: LoadState = { status: 'loading', config: null, isRunning: null }

export const InitialisationProvider = ({ children }: PropsWithChildren) => {
  const [loadState, setLoadState] = useState<LoadState>(INITIAL_STATE)
  const [isTogglingServer, setIsTogglingServer] = useState(false)

  useEffect(() => {
    const controller = new AbortController()

    void (async () => {
      try {
        const [config, isRunning] = await Promise.all([loadConfig(), isServerRunning()])
        if (controller.signal.aborted) return
        setLoadState({ status: 'ready', config, isRunning })
      } catch {
        if (controller.signal.aborted) return
        setLoadState({ status: 'failed', config: null, isRunning: null })
      }
    })()

    return () => controller.abort()
  }, [])

  const setIsRunning = (value: boolean) =>
    setLoadState((prev) => (prev.status === 'ready' ? { ...prev, isRunning: value } : prev))

  const toggleServer = async () => {
    if (loadState.status !== 'ready') return
    const stopping = loadState.isRunning
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
    <AppContext
      value={{ ...loadState, isTogglingServer, setIsRunning, toggleServer } as AppContextValue}
    >
      {children}
    </AppContext>
  )
}
