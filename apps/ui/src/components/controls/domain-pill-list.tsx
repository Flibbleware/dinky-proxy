import { X } from 'lucide-react'

type Props = {
  domains: string[]
  onRemove: (domain: string) => void
}

const DomainPillList = ({ domains, onRemove }: Props) => {
  if (!domains.length) return null

  return (
    <div className="flex flex-wrap gap-1.5">
      {domains.map((domain) => (
        <span
          key={domain}
          className="inline-flex items-center gap-1 rounded-full border border-emerald-300 bg-slate-800 px-3 py-1 text-slate-200 text-xs"
        >
          {domain}
          <button
            type="button"
            onClick={() => onRemove(domain)}
            aria-label={`Remove ${domain}`}
            className="cursor-pointer rounded-full text-slate-400 outline-none hover:text-slate-200 focus-visible:ring-1 focus-visible:ring-emerald-300"
          >
            <X className="size-3" />
          </button>
        </span>
      ))}
    </div>
  )
}

export { DomainPillList }
