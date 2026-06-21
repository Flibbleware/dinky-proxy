import { zodResolver } from '@hookform/resolvers/zod'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { DomainPillInput } from '@/components/controls/domain-pill-input'
import { Field, FieldContent, FieldError, FieldLabel, FieldSet } from '@/components/controls/field'
import { Input } from '@/components/controls/input'
import { FormSection } from '@/components/forms/form-section'
import ConfigurationActions from './actions'
import AdvancedDrawer from './advanced-drawer'
import AdvancedConfigurationSection from './advanced-section'
import { configurationSchema } from './schema'
import type { ConfigurationFormRecord, ConfigurationValues } from './types'
import { createFieldHelper, createHandleValidSubmit, getFormDefaults } from './utils'

type Props = {
  initialValues: ConfigurationValues
}

const Configuration = ({ initialValues }: Props) => {
  const form = useForm<ConfigurationFormRecord>({
    resolver: zodResolver(configurationSchema),
    defaultValues: getFormDefaults(initialValues),
  })

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = form

  const createFieldProps = createFieldHelper(register)
  const [showAdvanced, setShowAdvanced] = useState(false)

  return (
    <form
      className="flex flex-col gap-6"
      onSubmit={handleSubmit(createHandleValidSubmit(form.reset))}
    >
      <AdvancedDrawer open={showAdvanced} onClose={() => setShowAdvanced(false)}>
        <AdvancedConfigurationSection field={createFieldProps} errors={errors} />
      </AdvancedDrawer>

      <FormSection>
        <FieldSet>
          <legend className="sr-only">Proxy settings</legend>
          <div className="grid grid-cols-[1fr_auto] items-start gap-3">
            <Field>
              <FieldLabel<ConfigurationFormRecord>
                htmlFor="proxyHost"
                hint="The remote proxy hostname / IP address"
              >
                Host
              </FieldLabel>
              <FieldContent>
                <Input
                  {...createFieldProps('proxyHost')}
                  type="text"
                  autoComplete="off"
                  aria-invalid={!!errors.proxyHost}
                />
                <FieldError
                  id="proxyHost-error"
                  errors={errors.proxyHost ? [errors.proxyHost] : undefined}
                />
              </FieldContent>
            </Field>

            <Field className="w-28">
              <FieldLabel<ConfigurationFormRecord> htmlFor="proxyPort">Port</FieldLabel>
              <FieldContent>
                <Input
                  {...createFieldProps('proxyPort', { valueAsNumber: true })}
                  type="number"
                  inputMode="numeric"
                  autoComplete="off"
                  aria-invalid={!!errors.proxyPort}
                />
                <FieldError
                  id="proxyPort-error"
                  errors={errors.proxyPort ? [errors.proxyPort] : undefined}
                />
              </FieldContent>
            </Field>
          </div>

          <div className="grid grid-cols-2 items-start gap-3">
            <Field>
              <FieldLabel<ConfigurationFormRecord> htmlFor="username">Username</FieldLabel>
              <FieldContent>
                <Input
                  {...createFieldProps('username')}
                  type="text"
                  autoComplete="off"
                  spellCheck={false}
                  aria-invalid={!!errors.username}
                />
                <FieldError
                  id="username-error"
                  errors={errors.username ? [errors.username] : undefined}
                />
              </FieldContent>
            </Field>

            <Field>
              <FieldLabel<ConfigurationFormRecord>
                htmlFor="password"
                hint="Stored securely in the keychain"
              >
                Password
              </FieldLabel>
              <FieldContent>
                <Input
                  {...createFieldProps('password')}
                  type="password"
                  autoComplete="new-password"
                  aria-invalid={!!errors.password}
                />
                <FieldError
                  id="password-error"
                  errors={errors.password ? [errors.password] : undefined}
                />
              </FieldContent>
            </Field>
          </div>

          <Field>
            <FieldLabel<ConfigurationFormRecord> htmlFor="bypassList">Domains</FieldLabel>
            <FieldContent>
              <DomainPillInput
                control={form.control}
                name="bypassList"
                aria-describedby="bypassList-error"
              />
              <FieldError
                id="bypassList-error"
                errors={errors.bypassList ? [errors.bypassList] : undefined}
              />
            </FieldContent>
          </Field>
        </FieldSet>
      </FormSection>

      <ConfigurationActions
        showAdvanced={showAdvanced}
        onToggleAdvanced={() => setShowAdvanced((v) => !v)}
        control={form.control}
      />
    </form>
  )
}

export default Configuration
