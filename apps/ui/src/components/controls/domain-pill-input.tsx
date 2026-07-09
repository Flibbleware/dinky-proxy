import { type KeyboardEvent, useRef, useState } from 'react'
import { type Control, type FieldPath, type FieldValues, useController } from 'react-hook-form'
import { normalizeDomain, parseDomains, serializeDomains } from '@/lib/domains'
import { cn } from '@/lib/utils'
import { DomainPillList } from './domain-pill-list'

type Props<TFieldValues extends FieldValues, TTransformedValues = TFieldValues> = {
  control: Control<TFieldValues, unknown, TTransformedValues>
  name: FieldPath<TFieldValues>
  id?: string
  'aria-describedby'?: string
}

const DomainPillInput = <TFieldValues extends FieldValues, TTransformedValues = TFieldValues>({
  control,
  name,
  id,
  'aria-describedby': ariaDescribedBy,
}: Props<TFieldValues, TTransformedValues>) => {
  const { field, fieldState } = useController({ control, name })
  const [inputValue, setInputValue] = useState('')
  const listRef = useRef<HTMLUListElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  const domains = field.value ? parseDomains(String(field.value)) : []

  const addDomain = () => {
    const domain = normalizeDomain(inputValue)
    if (!domain || domains.includes(domain)) {
      setInputValue('')
      return
    }
    field.onChange(serializeDomains([...domains, domain]))
    setInputValue('')
  }

  const removeDomain = (domain: string, index: number) => {
    field.onChange(serializeDomains(domains.filter((d) => d !== domain)))
    // The removed pill's button had focus; move it to the next pill (or the input when
    // none remain) once the DOM has updated, so keyboard users don't drop to <body>.
    requestAnimationFrame(() => {
      const buttons = listRef.current?.querySelectorAll('button')
      const next = buttons?.[Math.min(index, buttons.length - 1)]
      if (next) {
        next.focus()
      } else {
        inputRef.current?.focus()
      }
    })
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
          'rounded-md border border-input bg-transparent transition-[border-color] has-[input:focus-visible]:border-brand has-[input:disabled]:opacity-50',
          fieldState.invalid && 'border-destructive',
        )}
      >
        <input
          ref={inputRef}
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
          className="w-full bg-transparent px-3 py-2 text-sm outline-none placeholder:text-muted-foreground disabled:pointer-events-none"
        />
      </div>
      <DomainPillList ref={listRef} domains={domains} onRemove={removeDomain} />
    </div>
  )
}

export { DomainPillInput }
