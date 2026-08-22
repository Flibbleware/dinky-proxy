import { Eye, EyeOff } from 'lucide-react'
import { type KeyboardEvent, useState } from 'react'
import { type Control, type FieldPath, type FieldValues, useController } from 'react-hook-form'
import { cn } from '@/lib/utils'
import { normalizeDomain, parseDomains, serializeDomains } from '@/screens/configuration/domains'
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
  const [obscured, setObscured] = useState(false)

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

  const removeDomain = (domain: string) => {
    field.onChange(serializeDomains(domains.filter((d) => d !== domain)))
  }

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      addDomain()
    }
  }

  const ObscureIcon = obscured ? EyeOff : Eye

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <div
          className={cn(
            'flex-1 rounded-md border border-input bg-transparent transition-[border-color] has-[input:focus-visible]:border-brand',
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
        <button
          type="button"
          onClick={() => setObscured(!obscured)}
          aria-label={obscured ? 'Show domains' : 'Hide domains'}
          aria-pressed={obscured}
          className="-mr-2 flex w-9 shrink-0 cursor-pointer items-center justify-center self-stretch rounded-md text-slate-400 outline-none transition-colors hover:text-slate-200 focus-visible:text-slate-200 focus-visible:ring-[3px] focus-visible:ring-ring/50"
        >
          <ObscureIcon className="size-5" />
        </button>
      </div>
      <DomainPillList domains={domains} obscured={obscured} onRemove={removeDomain} />
    </div>
  )
}

export { DomainPillInput }
