import type { ComponentProps } from 'react'

import { cn } from '@/lib/utils'

const Input = ({ className, type, ...props }: ComponentProps<'input'>) => (
  <input
    type={type}
    className={cn(
      'h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs outline-none transition-[color,box-shadow] selection:bg-primary selection:text-primary-foreground file:inline-flex file:h-7 file:border-0 file:bg-transparent file:font-medium file:text-foreground file:text-sm placeholder:text-muted-foreground disabled:pointer-events-none disabled:opacity-50 md:text-sm',
      'focus-visible:border-brand',
      'aria-invalid:border-destructive aria-invalid:ring-destructive/40',
      className,
    )}
    {...props}
  />
)

export { Input }
