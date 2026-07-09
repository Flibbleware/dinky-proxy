import { createFileRoute } from '@tanstack/react-router'
import { Button } from '@/components/controls/button'
import Configuration from '@/screens/configuration'
import { useInitialisation } from '@/useInitialisation'

const ConfigurationPage = () => {
  const { state, retryInitialisation } = useInitialisation()

  return (
    <div className="mx-auto flex min-h-screen max-w-3xl flex-col gap-8 px-6 pt-7 pb-7">
      <header>
        <p className="text-[11px] text-brand/80 uppercase tracking-[0.35em]">DinkyProxy</p>
        <h1 className="mt-2 font-semibold text-3xl text-foreground md:text-4xl">Configuration</h1>
        <p className="mt-3 max-w-2xl text-slate-300 text-sm">
          Configure the proxy settings and domains you would like to funnel through the proxy.
        </p>
      </header>

      {state.status === 'loading' && (
        <p role="status" className="text-muted-foreground text-sm">
          Loading configuration…
        </p>
      )}

      {state.status === 'failed' && (
        <div role="alert" className="flex flex-col items-start gap-3">
          <p className="text-muted-foreground text-sm">
            Failed to load the configuration. Check that the app backend is running, then try again.
          </p>
          <Button type="button" onClick={retryInitialisation}>
            Try again
          </Button>
        </div>
      )}

      {state.status === 'ready' && <Configuration initialValues={state.config} />}
    </div>
  )
}

export const Route = createFileRoute('/')({
  component: ConfigurationPage,
})
