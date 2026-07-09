import type { Control } from 'react-hook-form'
import type { z } from 'zod'
import type { configurationSchema } from './schema'

export type ConfigurationFormRecord = z.input<typeof configurationSchema>

export type ConfigurationParsed = z.output<typeof configurationSchema>

export type ConfigurationControl = Control<ConfigurationFormRecord, unknown, ConfigurationParsed>
