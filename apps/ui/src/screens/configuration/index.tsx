import { zodResolver } from '@hookform/resolvers/zod'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { Button } from '@/components/controls/button'
import { FormSection } from '@/components/forms/form-section'
import ConfigurationActions from './actions'
import AdvancedConfigurationSection from './advanced-section'
import BasicConfigurationSection from './basic-section'
import ConfigurationHeader from './header'
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
      className="flex flex-col gap-8"
      onSubmit={handleSubmit(createHandleValidSubmit(form.reset))}
    >
      <ConfigurationHeader>
        <ConfigurationActions control={form.control} />
      </ConfigurationHeader>

      <div className="relative">
        {/* Floated out of flow so the toggle doesn't widen the header-to-fields gap. */}
        <Button
          type="button"
          onClick={() => setShowAdvanced((v) => !v)}
          aria-controls="configuration-fields"
          className="absolute -top-12 right-0"
        >
          {showAdvanced ? 'Basic' : 'Advanced'}
        </Button>

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
      </div>
    </form>
  )
}

export default Configuration
