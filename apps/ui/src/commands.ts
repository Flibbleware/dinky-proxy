import { invoke } from '@tauri-apps/api/core'

const Command = {
  LoadConfig: 'load_config_command',
  SaveConfig: 'save_config_command',
  StartServer: 'start_server_command',
  StopServer: 'stop_server_command',
  IsServerRunning: 'is_server_running_command',
} as const

export type ProxyProtocol = 'http' | 'socks5'

export type ConfigurationValues = {
  port: number
  bypassDomains: string[]
  proxyProtocol: ProxyProtocol
  proxyHost: string
  proxyPort: number
  pacServerPort: number
  username: string
  password: string
}

type SaveConfigResult = { path: string; restarted: boolean }

export const loadConfig = (): Promise<ConfigurationValues> =>
  invoke<ConfigurationValues>(Command.LoadConfig)

export const saveConfig = (payload: ConfigurationValues): Promise<SaveConfigResult> =>
  invoke<SaveConfigResult>(Command.SaveConfig, { payload })

export const startServer = (): Promise<void> => invoke<void>(Command.StartServer)

export const stopServer = (): Promise<void> => invoke<void>(Command.StopServer)

export const isServerRunning = (): Promise<boolean> => invoke<boolean>(Command.IsServerRunning)
