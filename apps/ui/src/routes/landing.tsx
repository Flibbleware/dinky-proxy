import { createFileRoute } from '@tanstack/react-router'
import { useInitialisation } from '@/useInitialisation'
import Configuration from '../screens/configuration'
import ConfigurationHeader from '../screens/configuration/header'

const Landing = () => {
  const { status, config } = useInitialisation()

  return (
    <div className="min-h-screen bg-slate-950 text-slate-50">
      <div className="mx-auto max-w-3xl px-6 pt-7 pb-7">
        {status === 'ready' ? <Configuration initialValues={config} /> : <ConfigurationHeader />}
      </div>
    </div>
  )
}

export const Route = createFileRoute('/landing')({
  component: Landing,
})
