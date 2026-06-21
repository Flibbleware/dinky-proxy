import type { ComponentProps } from 'react'
import { cn } from '@/lib/utils'

const FormSection = ({ className, ...props }: ComponentProps<'section'>) => (
  <section
    className={cn(
      'rounded-xl border border-slate-800 bg-slate-900/60 p-4 shadow-emerald-500/10 shadow-sm',
      className,
    )}
    {...props}
  />
)

export { FormSection }
