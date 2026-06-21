import * as TooltipPrimitive from '@radix-ui/react-tooltip'
import type { ComponentProps } from 'react'
import { cn } from '@/lib/utils'

const TooltipProvider = TooltipPrimitive.Provider
const Tooltip = TooltipPrimitive.Root
const TooltipTrigger = TooltipPrimitive.Trigger

const TooltipContent = ({
  className,
  sideOffset = 6,
  collisionPadding = 8,
  children,
  ...props
}: ComponentProps<typeof TooltipPrimitive.Content>) => (
  <TooltipPrimitive.Portal>
    <TooltipPrimitive.Content
      sideOffset={sideOffset}
      collisionPadding={collisionPadding}
      className={cn(
        'z-50 max-w-xs rounded-md border border-slate-600 bg-slate-700 px-3 py-1.5 text-slate-200 text-sm shadow-md',
        'fade-in-0 zoom-in-95 animate-in',
        'data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=closed]:animate-out',
        'data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2',
        'data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2',
        className,
      )}
      {...props}
    >
      {children}
      <TooltipPrimitive.Arrow className="fill-slate-700" width={11} height={5} />
    </TooltipPrimitive.Content>
  </TooltipPrimitive.Portal>
)

export { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger }
