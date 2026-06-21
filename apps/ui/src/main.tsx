import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { RouterProvider, createHashHistory, createRouter } from '@tanstack/react-router'
// run pnpm dev to generate
import { routeTree } from './routeTree.gen'
import { Toaster } from './components/controls/sonner'
import { TooltipProvider } from './components/controls/tooltip'
import { AppError, ErrorBoundary } from './components/error-boundary'
import { InitialisationProvider } from './InitialisationContext'

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

createRoot(document.getElementById('root')!).render(
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
