import type { FieldErrors } from 'react-hook-form'
import { Field, FieldContent, FieldError, FieldLabel, FieldSet } from '@/components/controls/field'
import { Input } from '@/components/controls/input'
import { Select, SelectOption } from '@/components/controls/select'
import { useSettingsLocked } from '@/useInitialisation'
import type { ConfigurationFormRecord } from './types'
import type { createFieldHelper } from './utils'

type Props = {
  field: ReturnType<typeof createFieldHelper<ConfigurationFormRecord>>
  errors: FieldErrors<ConfigurationFormRecord>
}

const AdvancedConfigurationSection = ({ field, errors }: Props) => {
  const disabled = useSettingsLocked()

  return (
    <FieldSet disabled={disabled}>
      <legend className="sr-only">Advanced proxy settings</legend>
      <Field>
        <FieldLabel<ConfigurationFormRecord>
          htmlFor="proxyProtocol"
          hint="Choose the upstream proxy type. Credentials are used for both."
        >
          Protocol
        </FieldLabel>
        <FieldContent>
          <Select {...field('proxyProtocol')} aria-invalid={!!errors.proxyProtocol}>
            <SelectOption value="http">HTTP</SelectOption>
            <SelectOption value="socks5">SOCKS5</SelectOption>
          </Select>
          <FieldError id="proxyProtocol-error" error={errors.proxyProtocol} />
        </FieldContent>
      </Field>
      <div className="grid grid-cols-2 items-start gap-3">
        <Field>
          <FieldLabel<ConfigurationFormRecord>
            htmlFor="port"
            hint="The port the local wrapper listens on (used in the PAC file)."
          >
            Local Server Port
          </FieldLabel>
          <FieldContent>
            <Input
              {...field('port')}
              type="number"
              inputMode="numeric"
              autoComplete="off"
              aria-invalid={!!errors.port}
            />
            <FieldError id="port-error" error={errors.port} />
          </FieldContent>
        </Field>

        <Field>
          <FieldLabel<ConfigurationFormRecord>
            htmlFor="pacServerPort"
            hint="The port serving the PAC file (http://localhost:<port>)."
          >
            PAC Server Port
          </FieldLabel>
          <FieldContent>
            <Input
              {...field('pacServerPort')}
              type="number"
              inputMode="numeric"
              autoComplete="off"
              aria-invalid={!!errors.pacServerPort}
            />
            <FieldError id="pacServerPort-error" error={errors.pacServerPort} />
          </FieldContent>
        </Field>
      </div>
    </FieldSet>
  )
}

export default AdvancedConfigurationSection
