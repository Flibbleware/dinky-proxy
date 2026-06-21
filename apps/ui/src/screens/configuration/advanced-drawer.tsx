import type { PropsWithChildren } from 'react'
import { Button } from '@/components/controls/button'
import { cn } from '@/lib/utils'

type Props = PropsWithChildren<{
  open: boolean
  onClose: () => void
}>

const AdvancedDrawer = ({ open, onClose, children }: Props) => (
  <>
    <div
      id="advanced-settings"
      role="dialog"
      aria-modal="true"
      aria-labelledby="advanced-settings-heading"
      onKeyDown={(e) => e.key === 'Escape' && onClose()}
      className={cn(
        'fixed inset-0 z-50 border-slate-800 border-b bg-slate-950 shadow-2xl transition-transform duration-300 ease-in-out',
        open ? 'translate-y-0' : '-translate-y-full',
      )}
    >
      <div className="h-full overflow-y-auto">
        <div className="mx-auto max-w-3xl px-6 pt-7 pb-8">
          <div className="mb-8 flex items-start justify-between">
            <div>
              <p className="text-[11px] text-emerald-200/80 uppercase tracking-[0.35em]">
                DinkyProxy
              </p>
              <h2 id="advanced-settings-heading" className="mt-2 font-semibold text-3xl text-white">
                Advanced settings
              </h2>
            </div>
          </div>
          {children}
          <div className="mt-6">
            <Button type="button" onClick={onClose}>
              Close
            </Button>
          </div>
        </div>
      </div>
    </div>
  </>
)

export default AdvancedDrawer
