import './style.css'

import { createRouter, RouterProvider } from '@tanstack/react-router'
import { ThemeProvider } from 'next-themes'
import ReactDOM from 'react-dom/client'
import { AppViewmodel, AppViewmodelContext } from './app.viewmodel'
import { Toaster } from './components/ui/sonner'
import { TooltipProvider } from './components/ui/tooltip'

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
  const appViewmodel = new AppViewmodel()

  root.render(
    <AppViewmodelContext.Provider value={appViewmodel}>
      <ThemeProvider enableSystem attribute="class" defaultTheme="system">
        <TooltipProvider>
          <RouterProvider router={router} />
          <Toaster />
        </TooltipProvider>
      </ThemeProvider>
    </AppViewmodelContext.Provider>,
  )
}
