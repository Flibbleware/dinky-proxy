import type { ComponentProps } from 'react'
import { cn } from '@/lib/utils'

const FormSection = ({ className, ...props }: ComponentProps<'section'>) => (
  <section
    className={cn(
      'rounded-xl border border-slate-800 bg-slate-900/60 p-4 shadow-sm shadow-emerald-500/10',
      className,
    )}
    {...props}
  />
)

export { FormSection }
