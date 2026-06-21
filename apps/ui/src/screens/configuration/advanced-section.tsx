import type { FieldErrors } from 'react-hook-form'
import { Field, FieldContent, FieldError, FieldLabel, FieldSet } from '@/components/controls/field'
import { Input } from '@/components/controls/input'
import { Select, SelectOption } from '@/components/controls/select'
import { FormSection } from '@/components/forms/form-section'
import type { ConfigurationFormRecord } from './types'
import type { createFieldHelper } from './utils'

type Props = {
  field: ReturnType<typeof createFieldHelper<ConfigurationFormRecord>>
  errors: FieldErrors<ConfigurationFormRecord>
}

const AdvancedConfigurationSection = ({ field, errors }: Props) => (
  <FormSection aria-labelledby="advanced-settings-heading">
    <FieldSet>
      <legend className="sr-only">Advanced proxy settings</legend>
      <h2 id="advanced-settings-heading" className="sr-only">
        Advanced settings
      </h2>
      <Field>
        <FieldLabel<ConfigurationFormRecord>
          htmlFor="proxyProtocol"
          hint="Choose the upstream proxy type. Credentials are used for both."
        >
          Proxy protocol
        </FieldLabel>
        <FieldContent>
          <Select {...field('proxyProtocol')} aria-invalid={!!errors.proxyProtocol}>
            <SelectOption value="http">HTTP</SelectOption>
            <SelectOption value="socks5">SOCKS5</SelectOption>
          </Select>
          <FieldError
            id="proxyProtocol-error"
            errors={errors.proxyProtocol ? [errors.proxyProtocol] : undefined}
          />
        </FieldContent>
      </Field>
      <div className="grid grid-cols-2 items-start gap-3">
        <Field>
          <FieldLabel<ConfigurationFormRecord>
            htmlFor="port"
            hint="The port the local wrapper listens on (used in the PAC file)."
          >
            Local Server port
          </FieldLabel>
          <FieldContent>
            <Input
              {...field('port', { valueAsNumber: true })}
              type="number"
              inputMode="numeric"
              autoComplete="off"
              aria-invalid={!!errors.port}
            />
            <FieldError id="port-error" errors={errors.port ? [errors.port] : undefined} />
          </FieldContent>
        </Field>

        <Field>
          <FieldLabel<ConfigurationFormRecord>
            htmlFor="pacServerPort"
            hint="The port serving the PAC file (http://localhost:<port>)."
          >
            PAC server port
          </FieldLabel>
          <FieldContent>
            <Input
              {...field('pacServerPort', { valueAsNumber: true })}
              type="number"
              inputMode="numeric"
              autoComplete="off"
              aria-invalid={!!errors.pacServerPort}
            />
            <FieldError
              id="pacServerPort-error"
              errors={errors.pacServerPort ? [errors.pacServerPort] : undefined}
            />
          </FieldContent>
        </Field>
      </div>

      <Field>
        <FieldLabel<ConfigurationFormRecord>
          htmlFor="networkTarget"
          hint="The network service name to apply proxy settings to (e.g. Wi-Fi)."
        >
          Network service
        </FieldLabel>
        <FieldContent>
          <Input
            {...field('networkTarget')}
            type="text"
            autoComplete="off"
            aria-invalid={!!errors.networkTarget}
          />
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
