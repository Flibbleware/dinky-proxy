import { X } from 'lucide-react'

type Props = {
  domains: string[]
  onRemove: (domain: string) => void
}

const DomainPillList = ({ domains, onRemove }: Props) => {
  if (!domains.length) return null

  return (
    // biome-ignore lint/a11y/noRedundantRoles: WebKit/VoiceOver drops list semantics when list-style is removed; role="list" restores them
    <ul role="list" className="flex flex-wrap gap-1.5">
      {domains.map((domain) => (
        <li
          key={domain}
          className="inline-flex items-center gap-1 rounded-full border border-brand bg-slate-800 px-3 py-1 text-slate-200 text-xs"
        >
          {domain}
          <button
            type="button"
            onClick={() => onRemove(domain)}
            aria-label={`Remove ${domain}`}
            className="cursor-pointer rounded-full text-slate-400 outline-none hover:text-slate-200 focus-visible:ring-1 focus-visible:ring-brand"
          >
            <X className="size-3" />
          </button>
        </li>
      ))}
    </ul>
  )
}

export { DomainPillList }
