import { createContext, useContext } from 'react'
import type { ConfigurationValues } from '@/commands'

export type LoadState =
  | { status: 'loading'; config: null; isRunning: null }
  | { status: 'ready'; config: ConfigurationValues; isRunning: boolean }
  | { status: 'failed'; config: null; isRunning: null }

export type AppContextValue = {
  state: LoadState
  isTogglingServer: boolean
  setIsRunning: (value: boolean) => void
  setConfig: (config: ConfigurationValues) => void
  retryInitialisation: () => void
  toggleServer: () => Promise<void>
}

export const AppContext = createContext<AppContextValue | null>(null)

export const useInitialisation = () => {
  const ctx = useContext(AppContext)
  if (!ctx) throw new Error('useInitialisation must be used within InitialisationProvider')
  return ctx
}

export const useSettingsLocked = () => useInitialisation().state.isRunning === true
