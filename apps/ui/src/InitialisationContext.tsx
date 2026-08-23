import { type PropsWithChildren, useEffect, useState } from 'react'
import { toast } from 'sonner'
import {
  isServerRunning,
  loadConfig,
  onServerRunningChanged,
  startServer,
  stopServer,
  type UnlistenFn,
} from '@/commands'
import { AppContext, type AppContextValue } from './useInitialisation'

type LoadState = Pick<AppContextValue, 'status' | 'config' | 'isRunning'>

const INITIAL_STATE: LoadState = { status: 'loading', config: null, isRunning: null }

const withIsRunning =
  (value: boolean) =>
  (prev: LoadState): LoadState =>
    prev.status === 'ready' ? { ...prev, isRunning: value } : prev

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

  useEffect(() => {
    let unlisten: UnlistenFn | undefined
    let cancelled = false

    void (async () => {
      try {
        const stop = await onServerRunningChanged((isRunning) =>
          setLoadState(withIsRunning(isRunning)),
        )
        if (cancelled) stop()
        else unlisten = stop
      } catch (error) {
        console.error('Failed to subscribe to server state changes', error)
      }
    })()

    return () => {
      cancelled = true
      unlisten?.()
    }
  }, [])

  const setIsRunning = (value: boolean) => setLoadState(withIsRunning(value))

  const toggleServer = async () => {
    if (loadState.status !== 'ready') return
    const stopping = loadState.isRunning
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
      value={{ ...loadState, isTogglingServer, setIsRunning, toggleServer } as AppContextValue}
    >
      {children}
    </AppContext>
  )
}
