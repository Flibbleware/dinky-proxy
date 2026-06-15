import type { ComponentProps } from 'react'
import { cn } from '@/lib/utils'

const Button = ({ className, ...props }: ComponentProps<'button'>) => (
  <button
    className={cn(
      "focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
      'bg-primary text-primary-foreground hover:bg-primary/90',
      'h-9 px-4 py-2 has-[>svg]:px-3',
      className,
    )}
    {...props}
  />
)

export { Button }
