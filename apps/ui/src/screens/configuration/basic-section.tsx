import type { Control, FieldErrors } from 'react-hook-form'
import { DomainPillInput } from '@/components/controls/domain-pill-input'
import { Field, FieldContent, FieldError, FieldLabel, FieldSet } from '@/components/controls/field'
import { Input } from '@/components/controls/input'
import type { ConfigurationFormRecord } from './types'
import type { createFieldHelper } from './utils'

type Props = {
  field: ReturnType<typeof createFieldHelper<ConfigurationFormRecord>>
  errors: FieldErrors<ConfigurationFormRecord>
  control: Control<ConfigurationFormRecord>
}

const BasicConfigurationSection = ({ field, errors, control }: Props) => (
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
            {...field('proxyHost')}
            type="text"
            autoComplete="off"
            aria-invalid={!!errors.proxyHost}
          />
          <FieldError id="proxyHost-error" error={errors.proxyHost} />
        </FieldContent>
      </Field>

      <Field className="w-28">
        <FieldLabel<ConfigurationFormRecord> htmlFor="proxyPort">Port</FieldLabel>
        <FieldContent>
          <Input
            {...field('proxyPort', { valueAsNumber: true })}
            type="number"
            inputMode="numeric"
            autoComplete="off"
            aria-invalid={!!errors.proxyPort}
          />
          <FieldError id="proxyPort-error" error={errors.proxyPort} />
        </FieldContent>
      </Field>
    </div>

    <div className="grid grid-cols-2 items-start gap-3">
      <Field>
        <FieldLabel<ConfigurationFormRecord> htmlFor="username">Username</FieldLabel>
        <FieldContent>
          <Input
            {...field('username')}
            type="text"
            autoComplete="off"
            spellCheck={false}
            aria-invalid={!!errors.username}
          />
          <FieldError id="username-error" error={errors.username} />
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
            {...field('password')}
            type="password"
            autoComplete="new-password"
            aria-invalid={!!errors.password}
          />
          <FieldError id="password-error" error={errors.password} />
        </FieldContent>
      </Field>
    </div>

    <Field>
      <FieldLabel<ConfigurationFormRecord> htmlFor="bypassList">Domains</FieldLabel>
      <FieldContent>
        <DomainPillInput control={control} name="bypassList" aria-describedby="bypassList-error" />
        <FieldError id="bypassList-error" error={errors.bypassList} />
      </FieldContent>
    </Field>
  </FieldSet>
)

export default BasicConfigurationSection
