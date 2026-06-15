import { Component, type ErrorInfo, type PropsWithChildren } from 'react'

type State = {
  error: Error | null
}

const AppError = ({ error, reset }: { error: Error; reset?: () => void }) => (
  <div className="flex min-h-screen flex-col items-center justify-center gap-4 p-8 text-center">
    <h1 className="text-xl font-semibold">Something went wrong</h1>
    <p className="text-muted-foreground max-w-md text-sm">{error.message}</p>
    {reset && (
      <button
        onClick={reset}
        className="bg-primary text-primary-foreground hover:bg-primary/90 focus-visible:ring-ring/50 rounded-md px-4 py-2 text-sm font-medium outline-none focus-visible:ring-[3px]"
      >
        Try again
      </button>
    )}
  </div>
)

class ErrorBoundary extends Component<PropsWithChildren, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[ErrorBoundary]', error, info.componentStack)
  }

  render() {
    if (this.state.error) {
      return <AppError error={this.state.error} reset={() => this.setState({ error: null })} />
    }
    return this.props.children
  }
}

export { ErrorBoundary, AppError }
