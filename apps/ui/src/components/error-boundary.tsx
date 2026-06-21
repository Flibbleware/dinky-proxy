import { Component, type ErrorInfo, type PropsWithChildren } from 'react'

type State = {
  error: Error | null
}

const AppError = ({ error, reset }: { error: Error; reset?: () => void }) => (
  <div className="flex min-h-screen flex-col items-center justify-center gap-4 p-8 text-center">
    <h1 className="font-semibold text-xl">Something went wrong</h1>
    <p className="max-w-md text-muted-foreground text-sm">{error.message}</p>
    {reset && (
      <button
        type="button"
        onClick={reset}
        className="rounded-md bg-primary px-4 py-2 font-medium text-primary-foreground text-sm outline-none hover:bg-primary/90 focus-visible:ring-[3px] focus-visible:ring-ring/50"
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

export { AppError, ErrorBoundary }
