import { X } from 'lucide-react'
import type { Ref } from 'react'

type Props = {
  domains: string[]
  onRemove: (domain: string, index: number) => void
  ref?: Ref<HTMLUListElement>
}

const DomainPillList = ({ domains, onRemove, ref }: Props) => {
  if (!domains.length) return null

  return (
    // biome-ignore lint/a11y/noRedundantRoles: WebKit/VoiceOver drops list semantics when list-style is removed; role="list" restores them
    <ul ref={ref} role="list" className="flex flex-wrap gap-1.5">
      {domains.map((domain, index) => (
        <li
          key={domain}
          className="inline-flex items-center gap-1 rounded-full border border-brand bg-secondary px-3 py-1 text-secondary-foreground text-xs"
        >
          {domain}
          <button
            type="button"
            onClick={() => onRemove(domain, index)}
            aria-label={`Remove ${domain}`}
            className="cursor-pointer rounded-full text-muted-foreground outline-none hover:text-foreground focus-visible:ring-1 focus-visible:ring-brand disabled:cursor-not-allowed disabled:opacity-50"
          >
            <X className="size-3" />
          </button>
        </li>
      ))}
    </ul>
  )
}

export { DomainPillList }
