import { useFormState } from 'react-hook-form'
import { Button } from '@/components/controls/button'
import { useInitialisation } from '@/useInitialisation'
import { configurationSchema } from './schema'
import type { ConfigurationControl } from './types'
import { getFormDefaults } from './utils'

type Props = {
  showAdvanced: boolean
  onToggleAdvanced: () => void
  control: ConfigurationControl
}

const ConfigurationActions = ({ showAdvanced, onToggleAdvanced, control }: Props) => {
  const { isSubmitting, isDirty } = useFormState({ control })
  const { state, isTogglingServer, toggleServer } = useInitialisation()

  const isRunning = state.status === 'ready' && state.isRunning
  // The server starts from the saved config, not the live form values, so gate on that.
  const hasValidSavedConfig =
    state.status === 'ready' && configurationSchema.safeParse(getFormDefaults(state.config)).success

  return (
    <div className="flex items-center justify-between gap-3">
      <Button
        type="button"
        onClick={onToggleAdvanced}
        aria-expanded={showAdvanced}
        aria-controls="configuration-fields"
      >
        {showAdvanced ? 'Basic' : 'Advanced'}
      </Button>
      <div className="flex items-center gap-3">
        <Button
          type="button"
          onClick={toggleServer}
          disabled={isTogglingServer || !hasValidSavedConfig}
          className={isRunning ? 'bg-destructive text-white hover:bg-destructive/90' : ''}
        >
          {isRunning ? 'Disable' : 'Enable'}
        </Button>
        <Button type="submit" disabled={isSubmitting || !isDirty}>
          {isSubmitting ? 'Saving...' : 'Save Configuration'}
        </Button>
      </div>
    </div>
  )
}

export default ConfigurationActions
