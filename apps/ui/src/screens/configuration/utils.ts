import type { FieldPath, RegisterOptions, UseFormRegister } from 'react-hook-form'
import { toast } from 'sonner'
import { type ConfigurationValues, saveConfig } from '@/commands'
import { parseDomains, serializeDomains } from '@/lib/domains'
import type { ConfigurationFormRecord, ConfigurationParsed } from './types'

export const createFieldHelper =
  <TFieldValues extends Record<string, unknown>>(register: UseFormRegister<TFieldValues>) =>
  <T extends FieldPath<TFieldValues>>(name: T, options?: RegisterOptions<TFieldValues, T>) => ({
    id: name,
    'aria-describedby': `${name}-error`,
    ...register(name, options),
  })

export const getFormDefaults = ({
  bypassDomains,
  ...values
}: ConfigurationValues): ConfigurationFormRecord => ({
  ...values,
  bypassList: serializeDomains(bypassDomains),
})

export const createHandleValidSubmit =
  (
    reset: (values: ConfigurationFormRecord) => void,
    onSaved: (config: ConfigurationValues) => void,
  ) =>
  async ({ bypassList, ...values }: ConfigurationParsed) => {
    const config: ConfigurationValues = { ...values, bypassDomains: parseDomains(bypassList) }

    try {
      await saveConfig(config)
      onSaved(config)
      toast.success('Configuration Saved')
      reset({ ...values, bypassList })
    } catch (error) {
      console.error('Failed to save configuration', error)
      toast.error('Failed to save configuration')
    }
  }
