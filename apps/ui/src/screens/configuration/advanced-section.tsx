import { type FieldErrors } from 'react-hook-form'
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldLabel,
  FieldSet,
} from '@/components/controls/field'
import { Input } from '@/components/controls/input'
import { type ConfigurationFormFields } from './types'
import { Select, SelectOption } from '@/components/controls/select'
import { FormSection } from '@/components/forms/form-section'
import { createFieldHelper } from './utils'

type AdvancedConfigurationSectionProps = {
  field: ReturnType<typeof createFieldHelper<ConfigurationFormFields>>
  errors: FieldErrors<ConfigurationFormFields>
}

const AdvancedConfigurationSection = ({ field, errors }: AdvancedConfigurationSectionProps) => (
  <FormSection aria-labelledby="advanced-settings-heading">
    <FieldSet>
      <legend className="sr-only">Advanced proxy settings</legend>
      <h2 id="advanced-settings-heading" className="sr-only">
        Advanced settings
      </h2>
      <Field>
        <FieldLabel<ConfigurationFormFields> htmlFor="proxyProtocol">Proxy protocol</FieldLabel>
        <FieldContent>
          <Select {...field('proxyProtocol')} aria-invalid={!!errors.proxyProtocol}>
            <SelectOption value="http">HTTP</SelectOption>
            <SelectOption value="socks5">SOCKS5</SelectOption>
          </Select>
          <FieldDescription id="proxyProtocol-description">
            Choose the upstream proxy type. Credentials are used for both.
          </FieldDescription>
          <FieldError
            id="proxyProtocol-error"
            errors={errors.proxyProtocol ? [errors.proxyProtocol] : undefined}
          />
        </FieldContent>
      </Field>
      <Field>
        <FieldLabel<ConfigurationFormFields> htmlFor="port">Local Server port</FieldLabel>
        <FieldContent>
          <Input
            {...field('port', { valueAsNumber: true })}
            type="number"
            inputMode="numeric"
            autoComplete="off"
            aria-invalid={!!errors.port}
          />
          <FieldDescription id="port-description">
            The port the local wrapper listens on (used in the PAC file).
          </FieldDescription>
          <FieldError id="port-error" errors={errors.port ? [errors.port] : undefined} />
        </FieldContent>
      </Field>

      <Field>
        <FieldLabel<ConfigurationFormFields> htmlFor="pacServerPort">PAC server port</FieldLabel>
        <FieldContent>
          <Input
            {...field('pacServerPort', { valueAsNumber: true })}
            type="number"
            inputMode="numeric"
            autoComplete="off"
            aria-invalid={!!errors.pacServerPort}
          />
          <FieldDescription id="pacServerPort-description">
            The port serving the PAC file (http://localhost:&lt;port&gt;).
          </FieldDescription>
          <FieldError
            id="pacServerPort-error"
            errors={errors.pacServerPort ? [errors.pacServerPort] : undefined}
          />
        </FieldContent>
      </Field>

      <Field>
        <FieldLabel<ConfigurationFormFields> htmlFor="networkTarget">Network service</FieldLabel>
        <FieldContent>
          <Input
            {...field('networkTarget')}
            type="text"
            autoComplete="off"
            aria-invalid={!!errors.networkTarget}
          />
          <FieldDescription id="networkTarget-description">
            The network service name to apply proxy settings to (e.g. Wi-Fi).
          </FieldDescription>
          <FieldError
            id="networkTarget-error"
            errors={errors.networkTarget ? [errors.networkTarget] : undefined}
          />
        </FieldContent>
      </Field>
    </FieldSet>
  </FormSection>
)

export default AdvancedConfigurationSection
