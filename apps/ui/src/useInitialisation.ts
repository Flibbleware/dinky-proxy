import { createContext, useContext } from 'react'
import type { ConfigurationValues } from '@/screens/configuration/types'

export type AppStatus = 'loading' | 'ready' | 'failed'

export type AppContextValue = {
  status: AppStatus
  config: ConfigurationValues
  isRunning: boolean | null
  isTogglingServer: boolean
  setIsRunning: (value: boolean) => void
  toggleServer: () => Promise<void>
}

export const AppContext = createContext<AppContextValue | null>(null)

export const useInitialisation = () => {
  const ctx = useContext(AppContext)
  if (!ctx) throw new Error('useInitialisation must be used within InitialisationProvider')
  return ctx
}
