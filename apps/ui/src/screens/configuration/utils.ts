import { toast } from 'sonner'
import type { FieldPath, RegisterOptions, UseFormRegister } from 'react-hook-form'
import { type ConfigurationFormFields, type ConfigurationFormValues } from './types'
import { configurationSchema } from './schema'
import type { ConfigurationFormProps } from '.'

export const createFieldHelper =
  <TFieldValues extends Record<string, unknown>>(register: UseFormRegister<TFieldValues>) =>
  <T extends FieldPath<TFieldValues>>(name: T, options?: RegisterOptions<TFieldValues, T>) => ({
    id: name,
    'aria-describedby': `${name}-description ${name}-error`,
    ...register(name, options),
  })

export const getFormDefaults = (
  initialValues: Partial<ConfigurationFormValues>,
): ConfigurationFormFields => ({
  port: initialValues.port ?? 8888,
  proxyProtocol: initialValues.proxyProtocol ?? 'http',
  proxyHost: initialValues.proxyHost ?? 'xx.xx.xx.xx',
  proxyPort: initialValues.proxyPort ?? 8080,
  pacServerPort: initialValues.pacServerPort ?? 8000,
  networkTarget: initialValues.networkTarget ?? 'Wi-Fi',
  username: initialValues.username ?? '',
  password: initialValues.password ?? '',
  bypassList: (initialValues.bypassDomains ?? []).join('\n'),
})

export const createHandleValidSubmit =
  (onSubmit: ConfigurationFormProps['onSubmit']) => async (values: ConfigurationFormFields) => {
    const validated = configurationSchema.parse(values)
    const bypassDomains = validated.bypassList
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean)

    if (!onSubmit) {
      toast.error('No submit handler configured')
      return
    }

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
