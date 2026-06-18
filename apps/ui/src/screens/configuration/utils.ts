import { toast } from 'sonner'
import type { FieldPath, RegisterOptions, UseFormRegister } from 'react-hook-form'
import { type ConfigurationFormRecord, type ConfigurationValues } from './types'
import { configurationSchema } from './schema'
import { invoke } from '@tauri-apps/api/core'

export const createFieldHelper =
  <TFieldValues extends Record<string, unknown>>(register: UseFormRegister<TFieldValues>) =>
  <T extends FieldPath<TFieldValues>>(
    name: T,
    options?: RegisterOptions<TFieldValues, T> & { describedBy?: boolean },
  ) => {
    // Only reference the description element for fields that actually render one,
    // otherwise aria-describedby points at an id that never exists.
    const { describedBy, ...registerOptions } = options ?? {}
    const describedByIds = [describedBy ? `${name}-description` : undefined, `${name}-error`]
      .filter(Boolean)
      .join(' ')

    return {
      id: name,
      'aria-describedby': describedByIds,
      ...register(name, registerOptions),
    }
  }

export const getFormDefaults = (initialValues: ConfigurationValues): ConfigurationFormRecord => ({
  ...initialValues,
  bypassList: initialValues.bypassDomains.join('\n'),
})

const onSubmit = async (values: ConfigurationValues) => {
  void invoke('save_config_command', { payload: values })
}

export const createHandleValidSubmit = async (values: ConfigurationFormRecord) => {
  const validated = configurationSchema.parse(values)
  const bypassDomains = validated.bypassList
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)

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
  } catch (error) {
    console.error('Failed to save configuration', error)
    toast.error('Failed to save configuration')
  }
}
