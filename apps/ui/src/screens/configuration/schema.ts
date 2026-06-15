import { z } from 'zod'

export const proxyProtocolSchema = z.enum(['http', 'socks5'])

export const configurationSchema = z.object({
  port: z
    .union([z.number(), z.string()])
    .transform((val) => (typeof val === 'string' ? Number(val) : val))
    .pipe(
      z
        .number()
        .int()
        .min(1, 'Port must be between 1 and 65535')
        .max(65535, 'Port must be between 1 and 65535'),
    ),
  proxyPort: z
    .union([z.number(), z.string()])
    .transform((val) => (typeof val === 'string' ? Number(val) : val))
    .pipe(
      z
        .number()
        .int()
        .min(1, 'Proxy port must be between 1 and 65535')
        .max(65535, 'Proxy port must be between 1 and 65535'),
    ),
  pacServerPort: z
    .union([z.number(), z.string()])
    .transform((val) => (typeof val === 'string' ? Number(val) : val))
    .pipe(
      z
        .number()
        .int()
        .min(1, 'PAC server port must be between 1 and 65535')
        .max(65535, 'PAC server port must be between 1 and 65535'),
    ),
  proxyProtocol: proxyProtocolSchema,
  proxyHost: z.string().trim().min(1, 'Proxy host is required'),
  networkTarget: z.string().trim().min(1, 'Network service is required'),
  username: z.string().trim().min(1, 'Proxy username is required'),
  password: z.string().trim().min(1, 'Proxy password is required'),
  bypassList: z
    .string()
    .trim()
    .min(1, 'Enter at least one domain')
    .refine((value) => {
      const entries = value
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean)

      return entries.length > 0
    }, 'Enter at least one domain'),
})
