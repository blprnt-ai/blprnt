import './style.css'

import { createRouter, RouterProvider } from '@tanstack/react-router'
import ReactDOM from 'react-dom/client'
import { Toaster } from './components/ui/sonner'
import { TooltipProvider } from './components/ui/tooltip'
import { AppModelProvider } from './hooks/use-app-model'
import { routeTree } from './routeTree.gen'

const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

const rootElement = document.getElementById('root')!
if (!rootElement.innerHTML) {
  const root = ReactDOM.createRoot(rootElement)
  root.render(
    <>
      <TooltipProvider>
        <AppModelProvider>
          <RouterProvider router={router} />
        </AppModelProvider>
      </TooltipProvider>

      <Toaster />
    </>,
  )
}
