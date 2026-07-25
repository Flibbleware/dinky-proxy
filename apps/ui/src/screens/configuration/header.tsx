import type { ReactNode } from 'react'

type Props = {
  actions?: ReactNode
}

const ConfigurationHeader = ({ actions }: Props) => (
  <header className="flex items-start justify-between gap-4">
    <div>
      <p className="text-[11px] text-emerald-200/80 uppercase tracking-[0.35em]">DinkyProxy</p>
      <h1 className="mt-2 font-semibold text-3xl text-white md:text-4xl">Configuration</h1>
      <p className="mt-3 max-w-2xl text-slate-300 text-sm">
        Configure the proxy settings and domains you would like to funnel through the proxy.
      </p>
    </div>
    {actions}
  </header>
)

export default ConfigurationHeader
