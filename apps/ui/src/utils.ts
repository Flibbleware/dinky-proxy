import type { ConfigurationValues } from './screens/configuration/types'

type FirstLoadConfig = ConfigurationValues | undefined

export const isValidConfiguration = (value?: unknown): value is ConfigurationValues => {
  const config = value as FirstLoadConfig

  if (!config?.port) return false

  return (
    typeof config.port === 'number' &&
    Array.isArray(config.bypassDomains) &&
    typeof config.proxyHost === 'string' &&
    typeof config.proxyPort === 'number' &&
    typeof config.pacServerPort === 'number' &&
    typeof config.networkTarget === 'string' &&
    typeof config.username === 'string' &&
    typeof config.password === 'string'
  )
}
