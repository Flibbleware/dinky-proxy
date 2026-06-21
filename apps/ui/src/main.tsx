import { createHashHistory, createRouter, RouterProvider } from '@tanstack/react-router'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { Toaster } from './components/controls/sonner'
import { TooltipProvider } from './components/controls/tooltip'
import { AppError, ErrorBoundary } from './components/error-boundary'
import { InitialisationProvider } from './InitialisationContext'
// run pnpm dev to generate
import { routeTree } from './routeTree.gen'

const router = createRouter({
  history: createHashHistory(),
  routeTree,
  defaultPreload: 'intent',
  defaultErrorComponent: ({ error }) => <AppError error={error} />,
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

const rootElement = document.getElementById('root')
if (!rootElement) throw new Error('Root element #root not found')

createRoot(rootElement).render(
  <StrictMode>
    <ErrorBoundary>
      <TooltipProvider delayDuration={300}>
        <InitialisationProvider>
          <RouterProvider router={router} />
          <Toaster />
        </InitialisationProvider>
      </TooltipProvider>
    </ErrorBoundary>
  </StrictMode>,
)
