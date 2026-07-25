import { type Control, useFormState, useWatch } from 'react-hook-form'
import { Button } from '@/components/controls/button'
import { useInitialisation } from '@/useInitialisation'
import { configurationSchema } from './schema'
import type { ConfigurationFormRecord } from './types'

type Props = {
  control: Control<ConfigurationFormRecord>
}

const serverLabel = (isRunning: boolean | null): string => {
  if (isRunning === null) return 'Loading...'
  return isRunning ? 'Disable' : 'Enable'
}

const ConfigurationActions = ({ control }: Props) => {
  const { isSubmitting, isDirty } = useFormState({ control })
  const { isRunning, isTogglingServer, toggleServer } = useInitialisation()

  const watchedValues = useWatch({ control })
  const isFormValid = configurationSchema.safeParse(watchedValues).success

  return (
    <div className="flex items-center gap-3">
      <Button type="submit" disabled={isSubmitting || !isDirty}>
        {isSubmitting ? 'Saving...' : 'Save'}
      </Button>
      <Button
        type="button"
        onClick={toggleServer}
        disabled={isRunning === null || isTogglingServer || !isFormValid}
        className={isRunning ? 'bg-red-800 hover:bg-red-700' : ''}
      >
        {serverLabel(isRunning)}
      </Button>
    </div>
  )
}

export default ConfigurationActions
