import './styles/index.css'

import { reactErrorHandler } from '@sentry/react'
import { QueryClientProvider } from '@tanstack/react-query'
import { Suspense } from 'react'
import { createPortal } from 'react-dom'
import ReactDOM from 'react-dom/client'
import { HotkeysProvider } from 'react-hotkeys-hook'
import { App } from './app'
import { SimpleLoader } from './components/atoms/simple-loader'
import { Toaster } from './components/atoms/sonner'
import { ErrorBoundary } from './components/organisms/error-boundary'
import { AppViewModelProvider } from './context/app-viewmodel-context'
import { ModifierKeyProvider } from './context/modifier-key-context'
import { TextSelectedProvider } from './context/text-selected-context'
import { UpdateProvider } from './context/update-context'
import { BlprntEventEnum, once } from './lib/events/lib'
import { initSentry } from './lib/sentry/init'
import { queryClient } from './lib/utils/query-client'

const backendReadyPromise = () => new Promise<void>((resolve) => once(BlprntEventEnum.BackendReady, () => resolve()))

interface AppLoadedProps {
  backendReady: Promise<void>
}

const AppLoaded = ({ backendReady }: AppLoadedProps) => {
  return (
    <AppViewModelProvider>
      {/* TODO: Move all these to AppStore */}
      <UpdateProvider>
        <HotkeysProvider>
          <ModifierKeyProvider>
            <TextSelectedProvider>
              <App backendReady={backendReady} />
            </TextSelectedProvider>
          </ModifierKeyProvider>
        </HotkeysProvider>
      </UpdateProvider>
    </AppViewModelProvider>
  )
}

const rootElement = document.getElementById('app')
if (rootElement && !rootElement.innerHTML) {
  const root = ReactDOM.createRoot(rootElement, {
    onCaughtError: reactErrorHandler(),
    onRecoverableError: reactErrorHandler(),
    onUncaughtError: reactErrorHandler(),
  })

  const isDev = import.meta.env.DEV
  if (!isDev) initSentry()

  const theme = localStorage.getItem('theme') ?? 'dark'
  document.documentElement.classList.toggle('dark', theme === 'dark')

  const handleError = () => location.reload()

  const backendReady = backendReadyPromise()

  root.render(
    <ErrorBoundary
      action={handleError}
      actionLabel="Reload App"
      title="Fatal Error"
      errorMessage={
        <div>This is most likely due to an issue with an upstream service, not blprnt. Please try again later.</div>
      }
    >
      {createPortal(<Toaster className="z-1000" position="top-right" />, document.body)}

      <Suspense fallback={<SimpleLoader withMessage={false} />}>
        <QueryClientProvider client={queryClient}>
          <AppLoaded backendReady={backendReady} />
        </QueryClientProvider>
      </Suspense>
    </ErrorBoundary>,
  )
}
