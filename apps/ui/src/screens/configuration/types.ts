import { z } from 'zod'
import type { configurationSchema, proxyProtocolSchema } from './schema'

export type ConfigurationFormRecord = z.input<typeof configurationSchema>

export type ConfigurationValues = {
  port: number
  bypassDomains: string[]
  proxyProtocol: z.infer<typeof proxyProtocolSchema>
  proxyHost: string
  proxyPort: number
  pacServerPort: number
  networkTarget: string
  username: string
  password: string
}
