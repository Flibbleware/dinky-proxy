import { zodResolver } from '@hookform/resolvers/zod'
import { useState } from 'react'
import { flushSync } from 'react-dom'
import { type FieldErrors, type FieldPath, useForm } from 'react-hook-form'
import type { ConfigurationValues } from '@/commands'
import { FormSection } from '@/components/forms/form-section'
import { useInitialisation } from '@/useInitialisation'
import ConfigurationActions from './actions'
import AdvancedConfigurationSection from './advanced-section'
import BasicConfigurationSection from './basic-section'
import { configurationSchema } from './schema'
import type { ConfigurationFormRecord, ConfigurationParsed } from './types'
import { createFieldHelper, createHandleValidSubmit, getFormDefaults } from './utils'

type Props = {
  initialValues: ConfigurationValues
}

const ADVANCED_FIELDS: readonly (keyof ConfigurationFormRecord)[] = [
  'proxyProtocol',
  'port',
  'pacServerPort',
]

const isAdvancedField = (name: string) => (ADVANCED_FIELDS as readonly string[]).includes(name)

const Configuration = ({ initialValues }: Props) => {
  const { setConfig } = useInitialisation()
  const form = useForm<ConfigurationFormRecord, unknown, ConfigurationParsed>({
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

  // If every error is in the hidden section, react-hook-form can't focus or show any of
  // them — swap sections first (synchronously, so the field is mounted), then focus.
  const revealHiddenErrors = (submitErrors: FieldErrors<ConfigurationFormRecord>) => {
    const names = Object.keys(submitErrors)
    const visibleSectionHasError = names.some((name) => isAdvancedField(name) === showAdvanced)
    if (visibleSectionHasError || names.length === 0) return

    flushSync(() => setShowAdvanced((value) => !value))
    form.setFocus(names[0] as FieldPath<ConfigurationFormRecord>)
  }

  return (
    <form
      className="flex flex-col gap-6"
      onSubmit={handleSubmit(createHandleValidSubmit(form.reset, setConfig), revealHiddenErrors)}
    >
      <FormSection id="configuration-fields" className="min-h-[324px]">
        {showAdvanced ? (
          <AdvancedConfigurationSection field={createFieldProps} errors={errors} />
        ) : (
          <BasicConfigurationSection
            field={createFieldProps}
            errors={errors}
            control={form.control}
          />
        )}
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
