import type { FieldPath, RegisterOptions, UseFormRegister } from 'react-hook-form'
import { toast } from 'sonner'
import { saveConfig } from '@/commands'
import { parseDomains, serializeDomains } from './domains'
import { configurationSchema } from './schema'
import type { ConfigurationFormRecord, ConfigurationValues } from './types'

export const createFieldHelper =
  <TFieldValues extends Record<string, unknown>>(register: UseFormRegister<TFieldValues>) =>
  <T extends FieldPath<TFieldValues>>(name: T, options?: RegisterOptions<TFieldValues, T>) => ({
    id: name,
    'aria-describedby': `${name}-error`,
    ...register(name, options),
  })

export const getFormDefaults = (initialValues: ConfigurationValues): ConfigurationFormRecord => ({
  ...initialValues,
  bypassList: serializeDomains(initialValues.bypassDomains),
})

const onSubmit = async (values: ConfigurationValues) => {
  void saveConfig(values)
}

export const createHandleValidSubmit =
  (reset: (values: ConfigurationFormRecord) => void) => async (values: ConfigurationFormRecord) => {
    const validated = configurationSchema.parse(values)
    const bypassDomains = parseDomains(validated.bypassList)

    try {
      await onSubmit({
        port: validated.port,
        bypassDomains,
        proxyProtocol: validated.proxyProtocol,
        proxyHost: validated.proxyHost,
        proxyPort: validated.proxyPort,
        pacServerPort: validated.pacServerPort,
        networkTarget: validated.networkTarget,
        username: validated.username,
        password: validated.password,
      })
      toast.success('Configuration saved')
      reset(values)
    } catch (error) {
      console.error('Failed to save configuration', error)
      toast.error('Failed to save configuration')
    }
  }
