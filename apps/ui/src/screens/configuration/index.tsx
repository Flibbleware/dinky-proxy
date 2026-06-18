import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { Button } from '@/components/controls/button'
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldLabel,
  FieldSet,
} from '@/components/controls/field'
import { Input } from '@/components/controls/input'
import { Textarea } from '@/components/controls/textarea'
import { type ConfigurationFormFields, type ConfigurationFormValues } from './types'
import AdvancedConfigurationSection from './advanced-section'
import { createFieldHelper, createHandleValidSubmit, getFormDefaults } from './utils'
import { FormSection } from '@/components/forms/form-section'
import { configurationSchema } from './schema'

type Props = {
  initialValues: ConfigurationFormValues
}

const Configuration = ({ initialValues }: Props) => {
  const form = useForm<ConfigurationFormFields>({
    resolver: zodResolver(configurationSchema),
    defaultValues: getFormDefaults(initialValues),
  })

  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting, isDirty },
  } = form

  const createFieldProps = createFieldHelper(register)
  const [showAdvanced, setShowAdvanced] = useState(false)

  return (
    <form className="flex flex-col gap-6" onSubmit={handleSubmit(createHandleValidSubmit)}>
      <div className={`grid gap-5 ${showAdvanced ? 'md:grid-cols-2' : ''}`}>
        <FormSection>
          <FieldSet>
            <legend className="sr-only">Proxy settings</legend>
            <div className="grid grid-cols-[1fr_auto] items-start gap-3">
              <Field>
                <FieldLabel<ConfigurationFormFields>
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
                <FieldLabel<ConfigurationFormFields> htmlFor="proxyPort">Port</FieldLabel>
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
                <FieldLabel<ConfigurationFormFields> htmlFor="username">Username</FieldLabel>
                <FieldContent>
                  <Input
                    {...createFieldProps('username')}
                    type="text"
                    autoComplete="off"
                    aria-invalid={!!errors.username}
                  />
                  <FieldError
                    id="username-error"
                    errors={errors.username ? [errors.username] : undefined}
                  />
                </FieldContent>
              </Field>

              <Field>
                <FieldLabel<ConfigurationFormFields>
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
              <FieldLabel<ConfigurationFormFields> htmlFor="bypassList">Domains</FieldLabel>
              <FieldContent>
                <Textarea
                  {...createFieldProps('bypassList', { describedBy: true })}
                  rows={5}
                  placeholder="localhost&#10;*.internal.company&#10;example.com"
                  aria-invalid={!!errors.bypassList}
                />
                <FieldError
                  id="bypassList-error"
                  errors={errors.bypassList ? [errors.bypassList] : undefined}
                />
                <FieldDescription id="bypassList-description">
                  Separate each domain with a new line
                </FieldDescription>
              </FieldContent>
            </Field>
          </FieldSet>
        </FormSection>

        {showAdvanced && (
          <div id="advanced-settings">
            <AdvancedConfigurationSection field={createFieldProps} errors={errors} />
          </div>
        )}
      </div>

      <div className="flex items-center justify-between gap-3">
        <button
          type="button"
          onClick={() => setShowAdvanced((v) => !v)}
          aria-expanded={showAdvanced}
          aria-controls="advanced-settings"
          className="focus-visible:ring-ring/50 rounded-sm text-xs text-slate-400 transition-colors outline-none hover:text-slate-200 focus-visible:ring-[3px]"
        >
          {showAdvanced ? 'Hide advanced settings' : 'Show advanced settings'}
        </button>
        <Button type="submit" disabled={isSubmitting || !isDirty}>
          {isSubmitting ? 'Saving...' : 'Save configuration'}
        </Button>
      </div>
    </form>
  )
}

export default Configuration
