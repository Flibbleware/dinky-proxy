import { type Control, useFormState, useWatch } from 'react-hook-form'
import { Button } from '@/components/controls/button'
import { useInitialisation } from '@/useInitialisation'
import { configurationSchema } from './schema'
import type { ConfigurationFormRecord } from './types'

type Props = {
  showAdvanced: boolean
  onToggleAdvanced: () => void
  control: Control<ConfigurationFormRecord>
}

const serverLabel = (isRunning: boolean | null, isTogglingServer: boolean): string => {
  if (isRunning === null) return 'Loading...'
  if (isTogglingServer) return isRunning ? 'Stopping...' : 'Starting...'
  return isRunning ? 'Stop server' : 'Start server'
}

const ConfigurationActions = ({ showAdvanced, onToggleAdvanced, control }: Props) => {
  const { isSubmitting, isDirty } = useFormState({ control })
  const { isRunning, isTogglingServer, toggleServer } = useInitialisation()

  const watchedValues = useWatch({ control })
  const isFormValid = configurationSchema.safeParse(watchedValues).success

  return (
    <div className="flex items-center justify-between gap-3">
      <Button
        type="button"
        onClick={onToggleAdvanced}
        aria-expanded={showAdvanced}
        aria-controls="advanced-settings"
      >
        Advanced settings
      </Button>
      <div className="flex items-center gap-3">
        <Button
          type="button"
          onClick={toggleServer}
          disabled={isRunning === null || isTogglingServer || !isFormValid}
          className={isRunning ? 'bg-red-800 hover:bg-red-700' : ''}
        >
          {serverLabel(isRunning, isTogglingServer)}
        </Button>
        <Button type="submit" disabled={isSubmitting || !isDirty}>
          {isSubmitting ? 'Saving...' : 'Save configuration'}
        </Button>
      </div>
    </div>
  )
}

export default ConfigurationActions
