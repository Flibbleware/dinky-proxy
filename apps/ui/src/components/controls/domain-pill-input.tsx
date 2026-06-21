import { type KeyboardEvent, useState } from 'react'
import { type Control, type FieldPath, type FieldValues, useController } from 'react-hook-form'
import { cn } from '@/lib/utils'
import { parseDomains, serializeDomains } from '@/screens/configuration/domains'
import { DomainPillList } from './domain-pill-list'

type Props<TFieldValues extends FieldValues> = {
  control: Control<TFieldValues>
  name: FieldPath<TFieldValues>
  id?: string
  'aria-describedby'?: string
}

const DomainPillInput = <TFieldValues extends FieldValues>({
  control,
  name,
  id,
  'aria-describedby': ariaDescribedBy,
}: Props<TFieldValues>) => {
  const { field, fieldState } = useController({ control, name })
  const [inputValue, setInputValue] = useState('')

  const domains = field.value ? parseDomains(String(field.value)) : []

  const addDomain = () => {
    const trimmed = inputValue.trim()
    if (!trimmed || domains.includes(trimmed)) {
      setInputValue('')
      return
    }
    field.onChange(serializeDomains([...domains, trimmed]))
    setInputValue('')
  }

  const removeDomain = (domain: string) => {
    field.onChange(serializeDomains(domains.filter((d) => d !== domain)))
  }

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      addDomain()
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <div
        className={cn(
          'rounded-md border border-input bg-transparent transition-[border-color] has-[input:focus-visible]:border-brand',
          fieldState.invalid && 'border-destructive',
        )}
      >
        <input
          id={id ?? name}
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={field.onBlur}
          autoComplete="off"
          spellCheck={false}
          placeholder="Press Enter to add a domain"
          aria-describedby={ariaDescribedBy}
          aria-invalid={fieldState.invalid}
          className="w-full bg-transparent px-3 py-2 text-sm outline-none placeholder:text-muted-foreground"
        />
      </div>
      <DomainPillList domains={domains} onRemove={removeDomain} />
    </div>
  )
}

export { DomainPillInput }
