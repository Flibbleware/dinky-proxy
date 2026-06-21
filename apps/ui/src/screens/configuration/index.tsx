import { zodResolver } from '@hookform/resolvers/zod'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { FormSection } from '@/components/forms/form-section'
import ConfigurationActions from './actions'
import AdvancedConfigurationSection from './advanced-section'
import BasicConfigurationSection from './basic-section'
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
      {/* The basic and advanced fields swap in place within the same container. Hidden
          fields are unmounted, but react-hook-form retains their values, so toggling back
          and forth (and submitting) preserves everything either side. The min-height
          reserves room for the taller (basic) view so the card and footer don't shift
          when the shorter advanced view loads in. */}
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
