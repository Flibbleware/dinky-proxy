import { z } from 'zod'

export const proxyProtocolSchema = z.enum(['http', 'socks5'])

const portSchema = (label: string) =>
  z
    .union([z.number(), z.string()])
    .transform((val) => (typeof val === 'string' ? Number(val) : val))
    .pipe(
      z
        .number()
        .int()
        .min(1, `${label} must be between 1 and 65535`)
        .max(65535, `${label} must be between 1 and 65535`),
    )

export const configurationSchema = z.object({
  port: portSchema('Port'),
  proxyPort: portSchema('Proxy port'),
  pacServerPort: portSchema('PAC Server Port'),
  proxyProtocol: proxyProtocolSchema,
  proxyHost: z.string().trim().min(1, 'Proxy host is required'),
  username: z.string().trim().min(1, 'Proxy username is required'),
  password: z.string().trim().min(1, 'Proxy password is required'),
  bypassList: z.string().trim().min(1, 'Enter at least one domain'),
})
