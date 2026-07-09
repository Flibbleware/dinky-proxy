import type { ComponentProps } from 'react'
import { cn } from '@/lib/utils'

const FormSection = ({ className, ...props }: ComponentProps<'section'>) => (
  <section
    className={cn('rounded-xl border bg-card/60 p-4 shadow-brand/10 shadow-sm', className)}
    {...props}
  />
)

export { FormSection }
