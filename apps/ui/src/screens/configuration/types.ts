import { z } from 'zod'
import type { configurationSchema, proxyProtocolSchema } from './schema'

export type ConfigurationFormFields = z.input<typeof configurationSchema>

export type ConfigurationFormValues = {
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
