import { createContext, useContext } from 'react'
import type { ConfigurationValues } from '@/screens/configuration/types'

export type AppStatus = 'loading' | 'ready' | 'failed'

type AppContextValueBase = {
  isTogglingServer: boolean
  setIsRunning: (value: boolean) => void
  toggleServer: () => Promise<void>
}

type AppContextValueLoading = {
  status: 'loading'
  config: ConfigurationValues | null
  isRunning: null
} & AppContextValueBase

type AppContextValueReady = {
  status: 'ready'
  config: ConfigurationValues
  isRunning: boolean
} & AppContextValueBase

type AppContextValueFailed = {
  status: 'failed'
  config: null
  isRunning: null
} & AppContextValueBase

export type AppContextValue = AppContextValueLoading | AppContextValueReady | AppContextValueFailed

export const AppContext = createContext<AppContextValue | null>(null)

export const useInitialisation = () => {
  const ctx = useContext(AppContext)
  if (!ctx) throw new Error('useInitialisation must be used within InitialisationProvider')
  return ctx
}
