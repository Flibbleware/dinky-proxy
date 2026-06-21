import type { ComponentProps } from 'react'
import { cn } from '@/lib/utils'

const Button = ({ className, ...props }: ComponentProps<'button'>) => (
  <button
    className={cn(
      "inline-flex shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-md font-medium text-sm outline-none transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0",
      'bg-primary text-primary-foreground hover:bg-primary/90',
      'h-9 px-4 py-2 has-[>svg]:px-3',
      className,
    )}
    {...props}
  />
)

export { Button }
